use std::alloc::{Layout, alloc as sys_alloc, dealloc as sys_dealloc};
use std::{mem, ptr};

pub const CHUNK_DATA_SIZE: usize = 4096;

#[inline]
pub const fn align_up(addr: usize, align: usize) -> usize {
	debug_assert!(align.is_power_of_two());
	(addr + align - 1) & !(align - 1)
}

#[repr(C)]
pub struct ChunkHeader {
	pub free_slot: *mut u8,
	pub allocate_count: usize,
	pub next_chunk: *mut ChunkHeader,
}

pub struct SlabAllocator<const SLAB_SIZE: usize> {
	pub first_chunk: *mut ChunkHeader,
}

impl<const SLAB_SIZE: usize> SlabAllocator<SLAB_SIZE> {
	pub const MAX_SLOTS: usize = CHUNK_DATA_SIZE / SLAB_SIZE;

	#[inline]
	pub const fn header_padded_size() -> usize {
		align_up(size_of::<ChunkHeader>(), SLAB_SIZE)
	}

	#[inline]
	const fn chunk_alloc_size() -> usize {
		Self::header_padded_size() + CHUNK_DATA_SIZE
	}

	#[inline]
	const fn chunk_align() -> usize {
		Self::max_usize(align_of::<ChunkHeader>(), SLAB_SIZE)
	}

	#[inline]
	const fn max_usize(a: usize, b: usize) -> usize {
		if a > b { a } else { b }
	}

	pub fn new() -> Self {
		assert!(
			SLAB_SIZE >= size_of::<*mut u8>(),
			"SLAB_SIZE ({SLAB_SIZE})はポインターサイズ ({})以上である必要があります",
			size_of::<*mut u8>()
		);
		assert!(
			SLAB_SIZE.is_power_of_two(),
			"SLAB_SIZE ({SLAB_SIZE})は2の冪乗である必要があります"
		);
		Self { first_chunk: ptr::null_mut() }
	}

	pub const fn new_unchecked() -> Self {
		Self { first_chunk: ptr::null_mut() }
	}

	unsafe fn alloc_chunk() -> *mut ChunkHeader {
		unsafe {
			let layout = Layout::from_size_align(Self::chunk_alloc_size(), Self::chunk_align())
				.expect("チャンクのレイアウト計算に失敗");

			let raw = sys_alloc(layout);
			if raw.is_null() {
				panic!("SlabAllocator<{SLAB_SIZE}>: チャンクの確保に失敗 (OutOfMemory)");
			}

			let chunk = raw as *mut ChunkHeader;
			let data_start = raw.add(Self::header_padded_size());

			(*chunk).free_slot = data_start;
			(*chunk).allocate_count = 0;
			(*chunk).next_chunk = ptr::null_mut();

			ptr::write_bytes(data_start, 0, CHUNK_DATA_SIZE);

			chunk
		}
	}

	unsafe fn dealloc_chunk(chunk: *mut ChunkHeader) {
		unsafe {
			let layout = Layout::from_size_align(Self::chunk_alloc_size(), Self::chunk_align())
				.expect("チャンクのレイアウト計算に失敗");
			sys_dealloc(chunk as *mut u8, layout);
		}
	}

	#[inline]
	const unsafe fn chunk_data_start(chunk: *mut ChunkHeader) -> *mut u8 {
		unsafe { (chunk as *mut u8).add(Self::header_padded_size()) }
	}

	pub unsafe fn alloc(&mut self) -> *mut u8 {
		unsafe {
			if self.first_chunk.is_null() {
				self.first_chunk = Self::alloc_chunk();
			}

			let mut chunk = self.first_chunk;
			loop {
				if (*chunk).allocate_count < Self::MAX_SLOTS {
					break;
				}

				if (*chunk).next_chunk.is_null() {
					let new_chunk = Self::alloc_chunk();
					(*chunk).next_chunk = new_chunk;
				}
				chunk = (*chunk).next_chunk;
			}

			let slot = (*chunk).free_slot;

			let embedded: *mut u8 = *(slot as *const *mut u8);

			if embedded.is_null() {
				(*chunk).free_slot = slot.add(SLAB_SIZE);
			} else {
				(*chunk).free_slot = embedded;
				ptr::write_bytes(slot, 0, SLAB_SIZE);
			}

			(*chunk).allocate_count += 1;
			slot
		}
	}

	pub unsafe fn free(&mut self, ptr: *mut u8) {
		unsafe {
			let mut chunk = self.first_chunk;

			while !chunk.is_null() {
				let data_start = Self::chunk_data_start(chunk);
				let data_end = data_start.add(CHUNK_DATA_SIZE);

				if ptr >= data_start && ptr < data_end {
					*(ptr as *mut *mut u8) = (*chunk).free_slot;
					(*chunk).free_slot = ptr;
					(*chunk).allocate_count -= 1;

					if (*chunk).allocate_count == Self::MAX_SLOTS - 1 {
						let next = (*chunk).next_chunk;
						if !next.is_null() && (*next).allocate_count == 0 {
							(*chunk).next_chunk = (*next).next_chunk;
							Self::dealloc_chunk(next);
						}
					}

					return;
				}

				chunk = (*chunk).next_chunk;
			}
		}

		panic!(
			"SlabAllocator<{SLAB_SIZE}>::free: ポインター {ptr:p}はこのアロケーターで確保されたポインターではありません"
		)
	}

	pub unsafe fn owns(&self, ptr: *mut u8) -> bool {
		unsafe {
			let mut chunk = self.first_chunk;
			while !chunk.is_null() {
				let data_start = Self::chunk_data_start(chunk);
				let data_end = data_start.add(CHUNK_DATA_SIZE);
				if ptr >= data_start && ptr < data_end {
					return true;
				}
				chunk = (*chunk).next_chunk;
			}
			false
		}
	}

	pub unsafe fn chunk_count(&self) -> usize {
		unsafe {
			let mut count = 0;
			let mut chunk = self.first_chunk;
			while !chunk.is_null() {
				count += 1;
				chunk = (*chunk).next_chunk;
			}
			count
		}
	}
}

impl<const SLAB_SIZE: usize> Default for SlabAllocator<SLAB_SIZE> {
	fn default() -> Self {
		Self::new()
	}
}

impl<const SLAB_SIZE: usize> Drop for SlabAllocator<SLAB_SIZE> {
	fn drop(&mut self) {
		unsafe {
			let mut chunk = self.first_chunk;
			while !chunk.is_null() {
				let next = (*chunk).next_chunk;
				Self::dealloc_chunk(chunk);
				chunk = next;
			}
		}
	}
}

pub struct HeapAllocator;

impl HeapAllocator {
	pub unsafe fn alloc(size: usize, align: usize) -> *mut u8 {
		unsafe {
			let layout = Layout::from_size_align(size, align)
				.expect("HeapAllocator::alloc: 不正なメモリレイアウト");
			let ptr = sys_alloc(layout);
			if ptr.is_null() {
				panic!("HeapAllocator: メモリ確保に失敗 (OutOfMemory)");
			}
			ptr
		}
	}

	pub unsafe fn free(ptr: *mut u8, size: usize, align: usize) {
		unsafe {
			let layout = Layout::from_size_align(size, align)
				.expect("HeapAllocator::free: 不正なメモリレイアウト");
			sys_dealloc(ptr, layout);
		}
	}
}

#[inline]
fn slab_class(size: usize, align: usize) -> Option<usize> {
	let required = size.max(align);
	match required {
		1..=8 => Some(8),
		9..=16 => Some(16),
		17..=32 => Some(32),
		33..=64 => Some(64),
		_ => None,
	}
}

pub struct GeneralStorage {
	slab8: SlabAllocator<8>,
	slab16: SlabAllocator<16>,
	slab32: SlabAllocator<32>,
	slab64: SlabAllocator<64>,
}

impl GeneralStorage {
	pub const fn new() -> Self {
		Self {
			slab8: SlabAllocator::new_unchecked(),
			slab16: SlabAllocator::new_unchecked(),
			slab32: SlabAllocator::new_unchecked(),
			slab64: SlabAllocator::new_unchecked(),
		}
	}
}

impl Default for GeneralStorage {
	fn default() -> Self {
		Self::new()
	}
}

impl GeneralStorage {
	pub fn alloc_uninit<T: ?Sized, C, F>(&mut self, coerce: F) -> *mut T
	where
		C: Sized,
		F: FnOnce(*mut C) -> *mut T,
	{
		let size = size_of::<C>();
		let align = align_of::<C>();

		if size == 0 {
			// SAFETY: ZST の場合コンパイラはalign以上のダングリングポインターを許容するものとする
			let dangling = ptr::NonNull::<C>::dangling().as_ptr();
			return coerce(dangling);
		}

		unsafe {
			// スロットの確保を行う
			let raw: *mut u8 = self.alloc_raw(size, align);
			let dst: *mut C = raw.cast::<C>();

			// fat pointerを返す
			coerce(dst)
		}
	}
	/// コンクリート型 `C` の値をアロケーターメモリに直接配置し、その可変生ポインター `*mut T` を返す
	///
	/// # 型パラメーター
	/// - `C`: スタック上にある具体的な型。`size_of::<C>()` と `align_of::<C>()` を
	///   使ってどの Slabを使うかを決定する
	///
	/// # 引数
	/// - `value`: `C` スタックからアロケーターメモリへ直接ムーブされる
	/// - `coerce`: `*mut C → *mut T` への変換
	///   - `T == C` (Sized 型) の場合: `|p| p`
	///   - `T = dyn Trait` の場合: `|p| p as *mut dyn Trait`
	///
	///   このクロージャはコンパイル時にインライン展開され、実行時コストはゼロ
	///
	///   現在、 `Unsize` 型がnightlyにしか存在しないため、明示的な変換を必要とする
	///
	/// # メモリフロー
	///
	/// スタック上の `value: C` を `ptr::write` を用いて直接ヒープに書き込む
	///
	/// # Panics
	/// メモリ確保に失敗した場合 (OutOfMemory)
	pub fn alloc_with<T: ?Sized, C, F>(&mut self, value: C, coerce: F) -> *mut T
	where
		C: Sized,
		F: FnOnce(*mut C) -> *mut T,
	{
		let size = size_of::<C>();
		let align = align_of::<C>();

		if size == 0 {
			// ZSTの場合、実メモリ確保は不要なので `mem::forget` でデストラクターを抑制しつつ値を捨てる
			// danglingなnon-nullポインターを返す(RustのZST慣習に従う)
			mem::forget(value);
			// SAFETY: ZST の場合コンパイラはalign以上のダングリングポインターを許容するものとする
			let dangling = ptr::NonNull::<C>::dangling().as_ptr();
			return coerce(dangling);
		}

		unsafe {
			// スロットの確保を行う
			let raw: *mut u8 = self.alloc_raw(size, align);
			let dst: *mut C = raw.cast::<C>();

			// 値をスロットに直接配置する
			// `ptr::write` を用いてスタック上の `value` をヒープ上のスロットへムーブする
			// コピーコンストラクターもBoxも経由しない
			// TODO: 安全性チェック
			ptr::write(dst, value);

			// fat pointerを返す
			coerce(dst)
		}
	}

	/// `ptr` が指す値の型のデストラクターを実行し、対応するメモリを解放する
	///
	/// # Safety
	/// - `ptr` は同一アロケーターインスタンスの `alloc` / `alloc_with` が返したポインターであること
	/// - この呼び出し後、`ptr` を使用してはならない
	pub unsafe fn free<T: ?Sized>(&mut self, ptr: *mut T) {
		// drop_in_placeの後はvtable経由でサイズを読めなくなる可能性があるのでデストラクター実行前にレイアウト情報を確定させる
		unsafe {
			let size = size_of_val(&*ptr);
			let align = align_of_val(&*ptr);
			let data_ptr = ptr as *mut () as *mut u8;

			// デストラクター実行
			ptr::drop_in_place(ptr);

			if size == 0 {
				// ZSTは実メモリ確保なしなのでスキップ
				return;
			}

			self.free_raw(data_ptr, size, align);
		}
	}
}

// T: Sizedの場合の糖衣構文
// TODO: Unsize<T>がstableになったら廃止する
impl GeneralStorage {
	/// 値をアロケーターメモリに直接配置し、その可変生ポインターを返す
	///
	/// `alloc_with(value, |p| p)` の糖衣構文
	///
	/// # Examples
	/// ```rust
	/// let mut alloc: Storage<i32> = Storage::new();
	/// let ptr = alloc.alloc(-42i32);
	/// unsafe {
	///     assert_eq!(*ptr, -42);
	///     alloc.free(ptr);
	/// }
	/// ```
	#[inline]
	pub fn alloc<T: Sized>(&mut self, value: T) -> *mut T {
		self.alloc_with(value, |p| p)
	}
}

// 内部実装
impl GeneralStorage {
	/// `size`/`align` に合った Slab または Heap からメモリを確保する
	unsafe fn alloc_raw(&mut self, size: usize, align: usize) -> *mut u8 {
		unsafe {
			match slab_class(size, align) {
				Some(8) => self.slab8.alloc(),
				Some(16) => self.slab16.alloc(),
				Some(32) => self.slab32.alloc(),
				Some(64) => self.slab64.alloc(),
				_ => HeapAllocator::alloc(size, align),
			}
		}
	}

	/// `ptr` を適切な Slab または Heap に返却する
	pub unsafe fn free_raw(&mut self, ptr: *mut u8, size: usize, align: usize) {
		unsafe {
			match slab_class(size, align) {
				Some(8) => self.slab8.free(ptr),
				Some(16) => self.slab16.free(ptr),
				Some(32) => self.slab32.free(ptr),
				Some(64) => self.slab64.free(ptr),
				_ => HeapAllocator::free(ptr, size, align),
			}
		}
	}
}
