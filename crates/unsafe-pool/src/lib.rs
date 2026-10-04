use std::{
	alloc::{Layout, alloc, dealloc, handle_alloc_error},
	ptr::{self, NonNull},
};

/// 1チャンクの最大サイズ(バイト)。スロットがこれより大きい場合は1チャンク1スロットになる
const MAX_CHUNK_BYTES: usize = 64 * 1024;
/// 最初のチャンクのスロット数(上限は `MAX_CHUNK_BYTES` で頭打ち)
const INITIAL_CHUNK_SLOTS: usize = 16;

#[derive(Debug)]
struct Chunk {
	ptr: NonNull<u8>,
	layout: Layout,
}

/// 型消去された固定アドレスのプール
///
/// - `insert` が返すポインターは、`free` / `remove` / `release` されるまで**絶対に動かない**
///   (チャンクを追加するだけで、既存のチャンクは再配置しない)
/// - 空きスロットは全チャンクで共有される1本の侵入型フリーリストで管理する(LIFO)
/// - 確保は「フリーリストの pop」か「bump ポインターの前進」のみで、O(1)
/// - チャンクは `UnsafePool` の drop まで OS に返却しない
///
/// `UnsafePool` の drop は**生存中の要素のデストラクターを呼ばない**。
/// 呼び出し側が事前に `free` / `remove` しておく責任を持つ。
pub struct UnsafePool {
	/// フリーリストの先頭。空きスロットの先頭ポインターサイズ分に次の空きスロットが入っている
	free: *mut u8,
	/// 現在のチャンクの未使用領域の先頭
	bump: *mut u8,
	/// 現在のチャンクの終端
	bump_end: *mut u8,
	/// drop 用のチャンク一覧(スロット自体は参照しないので、この Vec の再配置は問題ない)
	chunks: Vec<Chunk>,
	slot_size: usize,
	slot_align: usize,
	/// 次に確保するチャンクのスロット数
	next_cap: usize,
	max_cap: usize,
	len: usize,
	#[cfg(debug_assertions)]
	type_name: &'static str,
	#[cfg(debug_assertions)]
	size: usize,
	#[cfg(debug_assertions)]
	align: usize,
}

const fn max_usize(a: usize, b: usize) -> usize {
	if a > b { a } else { b }
}

const fn min_usize(a: usize, b: usize) -> usize {
	if a < b { a } else { b }
}

impl UnsafePool {
	/// `T` 用のプールを作る。以降の `*_<T>` 系メソッドには同じ `T` を渡さなければならない
	#[must_use]
	pub fn new<T>() -> Self {
		// 空きスロットに次のポインターを書くので、ポインターが入る大きさ・アライメントを下限にする
		let slot_align = max_usize(align_of::<T>(), align_of::<*mut u8>());
		let raw_size = max_usize(size_of::<T>(), size_of::<*mut u8>());
		// スロットサイズを align の倍数に切り上げる(隣のスロットも必ず整列される)
		let slot_size = raw_size.div_ceil(slot_align) * slot_align;
		let max_cap = max_usize(1, MAX_CHUNK_BYTES / slot_size);
		Self {
			free: ptr::null_mut(),
			bump: ptr::null_mut(),
			bump_end: ptr::null_mut(),
			chunks: Vec::new(),
			slot_size,
			slot_align,
			next_cap: min_usize(INITIAL_CHUNK_SLOTS, max_cap),
			max_cap,
			len: 0,
			#[cfg(debug_assertions)]
			type_name: std::any::type_name::<T>(),
			#[cfg(debug_assertions)]
			size: size_of::<T>(),
			#[cfg(debug_assertions)]
			align: align_of::<T>(),
		}
	}

	/// 生存している要素数(確保済みで、まだ解放されていないスロット数)
	#[inline]
	#[must_use]
	pub const fn len(&self) -> usize {
		self.len
	}

	#[inline]
	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.len == 0
	}

	/// 確保済みチャンクの総スロット数
	#[inline]
	#[must_use]
	pub fn capacity(&self) -> usize {
		self.chunks.iter().map(|c| c.layout.size() / self.slot_size).sum()
	}

	#[inline]
	#[must_use]
	pub fn chunk_count(&self) -> usize {
		self.chunks.len()
	}

	/// `value` をプールに配置し、そのアドレスを返す
	///
	/// # Safety
	/// - `T` はこのインスタンスの構築時に使われた型と一致していなければならない
	/// - 返されたポインターが指す `T` が、以下の要件を満たすように使うこと
	///   - `free` / `remove` / `release` のいずれかを呼ぶまでは有効であり、その後は使用してはならない
	///   - 参照を作る場合、Rustの別名規則(`&mut T` の排他性など)を呼び出し側が守ること
	///   - 生存中に `UnsafePool` を drop する場合、デストラクターは呼ばれない
	#[inline]
	#[must_use]
	pub unsafe fn insert<T>(&mut self, value: T) -> *mut T {
		let slot = unsafe { self.reserve::<T>() };
		unsafe { slot.write(value) };
		slot
	}

	/// 値を書き込まずにスロットだけ確保し、アドレスを確定させる
	///
	/// 返されたポインターの指す先は**未初期化**。
	/// 利用前に `ptr.write(value)` で初期化するか、不要になったなら `release` で返却すること。
	///
	/// # Safety
	/// - `T` はこのインスタンスの構築時に使われた型と一致していなければならない
	/// - 返されたポインターは、`write` で初期化するまで読んではならない
	/// - その他は `insert` と同じ
	#[inline]
	#[must_use]
	pub unsafe fn reserve<T>(&mut self) -> *mut T {
		self.debug_assert_type::<T>();
		let slot = if !self.free.is_null() {
			let slot = self.free;
			// SAFETY: フリーリスト上のスロットは先頭に次のポインターを持つ
			self.free = unsafe { (slot as *const *mut u8).read() };
			slot
		} else {
			if self.bump == self.bump_end {
				self.grow();
			}
			let slot = self.bump;
			self.bump = unsafe { slot.add(self.slot_size) };
			slot
		};
		self.len += 1;
		slot as *mut T
	}

	/// 値のデストラクターを呼ばずに、スロットだけをフリーリストへ返却する
	///
	/// `reserve` したが使わなかった場合や、値を `ptr::read` で取り出した後に使う。
	///
	/// # Safety
	/// - `T` はこのインスタンスの構築時に使われた型と一致していなければならない
	/// - `ptr` はこのインスタンスの `insert` / `reserve` が返し、まだ返却されていないポインターであること(二重解放の禁止)
	/// - 値が初期化済みの場合、その破棄(もしくはムーブ)は呼び出し側で済ませていること
	/// - この呼び出し以降、`ptr` および `ptr` から作られた参照を使用してはならない
	#[inline]
	pub unsafe fn release<T>(&mut self, ptr: *mut T) {
		self.debug_assert_type::<T>();
		debug_assert!(self.len > 0, "UnsafePool: release called on an empty pool.");
		debug_assert!(
			self.owns(ptr as *const u8),
			"UnsafePool: pointer does not belong to this pool."
		);
		let slot = ptr as *mut u8;
		#[cfg(debug_assertions)]
		unsafe {
			// 解放後参照を検出しやすくするため、毒値で埋める
			ptr::write_bytes(slot, 0xDD, self.slot_size);
		}
		unsafe { (slot as *mut *mut u8).write(self.free) };
		self.free = slot;
		self.len -= 1;
	}

	/// 値をムーブして取り出し、スロットを返却する
	///
	/// # Safety
	/// - `release` の要件に加えて、`ptr` が指す `T` が初期化済みであること
	/// - 戻り値の `T` の破棄は呼び出し側の責任
	#[inline]
	#[must_use]
	pub unsafe fn remove<T>(&mut self, ptr: *mut T) -> T {
		let value = unsafe { ptr.read() };
		unsafe { self.release(ptr) };
		value
	}

	/// 値のデストラクターを実行し、スロットを返却する
	///
	/// `T::drop` の中でこのプールを触ってはならない(`&mut self` を保持しているため、
	/// 子を再帰的に drop する型の場合は `remove` で取り出してから drop すること)。
	///
	/// # Safety
	/// - `remove` と同じ
	#[inline]
	pub unsafe fn free<T>(&mut self, ptr: *mut T) {
		unsafe { ptr::drop_in_place(ptr) };
		unsafe { self.release(ptr) };
	}

	/// `ptr` がこのプールのチャンク内のアドレスかどうか(O(チャンク数)、debug 用途)
	#[must_use]
	pub fn owns(&self, ptr: *const u8) -> bool {
		let addr = ptr as usize;
		self.chunks.iter().any(|c| {
			let start = c.ptr.as_ptr() as usize;
			let end = start + c.layout.size();
			addr >= start && addr < end && (addr - start).is_multiple_of(self.slot_size)
		})
	}

	#[cold]
	#[inline(never)]
	fn grow(&mut self) {
		let cap = self.next_cap;
		let size = self.slot_size.checked_mul(cap).expect("UnsafePool: capacity overflow");
		let layout =
			Layout::from_size_align(size, self.slot_align).expect("UnsafePool: layout error");
		// SAFETY: size > 0 (slot_size >= ポインターサイズ、cap >= 1)
		let raw = unsafe { alloc(layout) };
		let Some(raw_nn) = NonNull::new(raw) else { handle_alloc_error(layout) };
		self.chunks.push(Chunk { ptr: raw_nn, layout });
		self.bump = raw;
		self.bump_end = unsafe { raw.add(size) };
		self.next_cap = min_usize(cap.saturating_mul(2), self.max_cap);
	}

	#[cfg(debug_assertions)]
	#[inline]
	fn debug_assert_type<T>(&self) {
		debug_assert!(
			self.size == size_of::<T>() && self.align == align_of::<T>(),
			"UnsafePool: type mismatch (pool is for `{}`, called with `{}`).",
			self.type_name,
			std::any::type_name::<T>()
		);
	}

	#[cfg(not(debug_assertions))]
	#[inline(always)]
	fn debug_assert_type<T>(&self) {}
}

impl Drop for UnsafePool {
	fn drop(&mut self) {
		// 生存中の要素のデストラクターは呼ばない(呼び出し側の責任)
		for chunk in self.chunks.drain(..) {
			unsafe { dealloc(chunk.ptr.as_ptr(), chunk.layout) };
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{cell::Cell, rc::Rc};

	use super::*;

	#[test]
	fn insert_and_read() {
		let mut pool = UnsafePool::new::<u32>();
		let a = unsafe { pool.insert(5u32) };
		let b = unsafe { pool.insert(8u32) };
		assert_eq!(unsafe { *a }, 5);
		assert_eq!(unsafe { *b }, 8);
		assert_eq!(pool.len(), 2);
		unsafe { *a += 1 };
		assert_eq!(unsafe { *a }, 6);
	}

	#[test]
	fn freed_slots_are_reused_in_lifo_order() {
		let mut pool = UnsafePool::new::<u64>();
		let ptrs: Vec<*mut u64> = (0..6u64).map(|v| unsafe { pool.insert(v * 10) }).collect();
		assert_eq!(unsafe { pool.remove(ptrs[2]) }, 20);
		assert_eq!(unsafe { pool.remove(ptrs[4]) }, 40);
		assert_eq!(unsafe { pool.remove(ptrs[1]) }, 10);
		assert_eq!(pool.len(), 3);
		let r1 = unsafe { pool.insert(100u64) };
		let r2 = unsafe { pool.insert(200u64) };
		let r3 = unsafe { pool.insert(300u64) };
		assert_eq!((r1, r2, r3), (ptrs[1], ptrs[4], ptrs[2]));
		assert_eq!(unsafe { *ptrs[0] }, 0);
		assert_eq!(unsafe { *ptrs[3] }, 30);
		assert_eq!(unsafe { *ptrs[5] }, 50);
	}

	#[test]
	fn addresses_are_stable_across_growth() {
		let mut pool = UnsafePool::new::<u64>();
		const N: u64 = 100_000;
		let ptrs: Vec<*mut u64> = (0..N).map(|i| unsafe { pool.insert(i) }).collect();
		assert!(pool.chunk_count() > 1);
		assert!(pool.capacity() >= N as usize);
		for (i, p) in ptrs.iter().enumerate() {
			assert_eq!(unsafe { **p }, i as u64, "slot {i} was corrupted or moved");
		}
		// すべて異なるアドレス
		let mut addrs: Vec<usize> = ptrs.iter().map(|p| *p as usize).collect();
		addrs.sort_unstable();
		addrs.dedup();
		assert_eq!(addrs.len(), N as usize);
	}

	#[test]
	fn reserve_gives_address_before_value() {
		struct Node {
			self_ptr: *mut Node,
			value: u32,
		}
		let mut pool = UnsafePool::new::<Node>();
		let p = unsafe { pool.reserve::<Node>() };
		unsafe { p.write(Node { self_ptr: p, value: 42 }) };
		let node = unsafe { &*p };
		assert_eq!(node.self_ptr, p);
		assert_eq!(node.value, 42);
		assert_eq!(pool.len(), 1);
	}

	#[test]
	fn release_returns_reserved_slot() {
		let mut pool = UnsafePool::new::<u32>();
		let p = unsafe { pool.reserve::<u32>() };
		assert_eq!(pool.len(), 1);
		unsafe { pool.release(p) };
		assert!(pool.is_empty());
		let q = unsafe { pool.insert(7u32) };
		assert_eq!(p, q);
		assert_eq!(unsafe { *q }, 7);
	}

	#[test]
	fn free_runs_destructor_and_drop_does_not() {
		let counter = Rc::new(());
		let mut pool = UnsafePool::new::<Rc<()>>();
		let a = unsafe { pool.insert(counter.clone()) };
		let _b = unsafe { pool.insert(counter.clone()) };
		assert_eq!(Rc::strong_count(&counter), 3);
		unsafe { pool.free(a) };
		assert_eq!(Rc::strong_count(&counter), 2);
		// プールの drop は生存要素を破棄しない(リークが仕様)
		drop(pool);
		assert_eq!(Rc::strong_count(&counter), 2);
	}

	#[test]
	fn respects_large_alignment() {
		#[repr(align(64))]
		struct Aligned64(u64);
		let mut pool = UnsafePool::new::<Aligned64>();
		let ptrs: Vec<*mut Aligned64> =
			(0..50u64).map(|i| unsafe { pool.insert(Aligned64(i)) }).collect();
		for (i, p) in ptrs.iter().enumerate() {
			assert_eq!(*p as usize % 64, 0);
			assert_eq!(unsafe { (**p).0 }, i as u64);
		}
	}

	#[test]
	fn small_and_zero_sized_types() {
		let mut pool = UnsafePool::new::<u8>();
		let ptrs: Vec<*mut u8> = (0..100u8).map(|i| unsafe { pool.insert(i) }).collect();
		for (i, p) in ptrs.iter().enumerate() {
			assert_eq!(unsafe { **p }, i as u8);
		}
		let mut zst_pool = UnsafePool::new::<()>();
		let a = unsafe { zst_pool.insert(()) };
		let b = unsafe { zst_pool.insert(()) };
		assert_ne!(a, b);
	}

	#[test]
	fn huge_type_gets_one_slot_per_chunk() {
		struct Huge([u8; 100_000]);
		let mut pool = UnsafePool::new::<Huge>();
		let a = unsafe { pool.insert(Huge([1; 100_000])) };
		let b = unsafe { pool.insert(Huge([2; 100_000])) };
		assert_eq!(pool.chunk_count(), 2);
		assert_eq!(unsafe { (*a).0[99_999] }, 1);
		assert_eq!(unsafe { (*b).0[0] }, 2);
	}

	#[test]
	fn insert_during_use_of_existing_reference_is_sound() {
		// view(&self) 中に他の要素を insert しても、既存要素が動かないことの確認
		let mut pool = UnsafePool::new::<String>();
		let first = unsafe { pool.insert(String::from("hello")) };
		let first_ref: &String = unsafe { &*first };
		for i in 0..10_000 {
			let _ = unsafe { pool.insert(i.to_string()) };
		}
		assert_eq!(first_ref, "hello");
		assert_eq!(first, first_ref as *const String as *mut String);
	}

	#[test]
	fn mixed_workload_keeps_len_consistent() {
		let mut pool = UnsafePool::new::<u32>();
		let live = Cell::new(0usize);
		let mut ptrs = Vec::new();
		for round in 0..50u32 {
			for i in 0..200u32 {
				ptrs.push(unsafe { pool.insert(round * 1000 + i) });
				live.set(live.get() + 1);
			}
			for _ in 0..150 {
				let p = ptrs.swap_remove((round as usize * 7) % ptrs.len());
				let _ = unsafe { pool.remove(p) };
				live.set(live.get() - 1);
			}
			assert_eq!(pool.len(), live.get());
		}
	}

	#[test]
	#[cfg(debug_assertions)]
	#[should_panic(expected = "type mismatch")]
	fn type_mismatch_is_caught_in_debug() {
		let mut pool = UnsafePool::new::<u32>();
		let _ = unsafe { pool.insert(1u64) };
	}
}
