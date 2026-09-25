use std::num::NonZeroUsize;

use crate::storage::unsafe_vec::UnsafeVec;

enum Slot<T> {
	Occupied(T),
	Empty(usize),
}

impl<T> Slot<T> {
	#[inline]
	const fn occupied(&self) -> bool {
		matches!(self, Slot::Occupied(_))
	}
}

#[derive(Debug)]
pub struct UnsafeSlotMap {
	slots: UnsafeVec,
	free_head: usize,
	num_elems: usize,
}

impl UnsafeSlotMap {
	pub const fn new_for<T>() -> Self {
		let slots = UnsafeVec::new_for::<Slot<T>>();

		Self { slots, free_head: 0, num_elems: 0 }
	}

	pub fn with_capacity_for<T>(capacity: NonZeroUsize) -> Self {
		let slots = unsafe { UnsafeVec::with_capacity_for::<T>(capacity) };

		Self { slots, free_head: 0, num_elems: 0 }
	}

	#[inline]
	pub const fn len(&self) -> usize {
		self.num_elems
	}

	#[inline]
	pub const fn is_empty(&self) -> bool {
		self.num_elems == 0
	}

	#[inline]
	#[must_use]
	pub unsafe fn insert_for<T>(&mut self, value: T) -> usize {
		let slot_idx = self.free_head;
		let inner_len = self.slots.len();

		if self.num_elems < inner_len {
			let slot = unsafe { self.slots.get_mut_for::<Slot<T>>(slot_idx) };
			let next_free = match slot {
				Slot::Empty(next_free) => *next_free,
				Slot::Occupied(_) => unreachable!(),
			};
			*slot = Slot::Occupied(value);
			self.free_head = next_free;
		} else {
			unsafe { self.slots.push_for(Slot::Occupied(value)) };
			self.free_head = self.num_elems + 1;
		}

		self.num_elems += 1;
		slot_idx
	}

	/// # Safety
	/// `idx`は現在occupiedなスロットのインデックスである必要がある
	/// `T`はこのインスタンスの構築時に使われた型と一致していなければならない
	#[inline]
	#[must_use]
	pub unsafe fn get_for<T>(&self, idx: usize) -> &T {
		let slot = unsafe { self.slots.get_for::<Slot<T>>(idx) };

		match slot {
			Slot::Empty(_) => unreachable!(),
			Slot::Occupied(value) => value,
		}
	}

	/// # Safety
	/// `idx`は現在occupiedなスロットのインデックスである必要がある
	/// `T`はこのインスタンスの構築時に使われた型と一致していなければならない
	///
	/// `T`がデストラクタを保つ場合、戻り値の`T`の破棄は呼び出し側の責任となる
	#[inline]
	#[must_use]
	pub unsafe fn remove_for<T>(&mut self, idx: usize) -> T {
		let slot = unsafe { self.slots.get_mut_for::<Slot<T>>(idx) };
		let slot_data = std::mem::replace(slot, Slot::Empty(self.free_head));
		self.free_head = idx;
		self.num_elems -= 1;

		match slot_data {
			Slot::Empty(_) => unreachable!(),
			Slot::Occupied(value) => value,
		}
	}
}

#[test]
pub fn unsafe_slot_map() {
	let mut slot_map = UnsafeSlotMap::new_for::<u32>();
	let idx1 = unsafe { slot_map.insert_for::<u32>(5) };
	let idx2 = unsafe { slot_map.insert_for::<u32>(8) };
	assert!(idx1 == 0);
	assert!(idx2 == 1);

	let value1 = unsafe { slot_map.get_for::<u32>(idx1) };
	let value2 = unsafe { slot_map.get_for::<u32>(idx2) };
	assert!(*value1 == 5);
	assert!(*value2 == 8);

	let _ = unsafe { slot_map.remove_for::<u32>(idx1) };

	let idx1_new = unsafe { slot_map.insert_for::<u32>(9) };
	assert!(idx1 == idx1_new);
}

#[test]
fn caller_is_responsible_for_dropping_removed_elements() {
	use std::rc::Rc;

	let counter = Rc::new(());
	let mut slot_map = UnsafeSlotMap::new_for::<Rc<()>>();
	let idx1 = unsafe { slot_map.insert_for(counter.clone()) };
	let idx2 = unsafe { slot_map.insert_for(counter.clone()) };
	assert_eq!(Rc::strong_count(&counter), 3);

	// Safety: Rc<()>はデストラクタを持つため、破棄前にremove_forで
	// 全要素を回収し、呼び出し側の責任でdropする
	let _ = unsafe { slot_map.remove_for::<Rc<()>>(idx1) };
	let _ = unsafe { slot_map.remove_for::<Rc<()>>(idx2) };
	assert_eq!(Rc::strong_count(&counter), 1);

	// ここでdropされる内部バッファ解放は要素の有無に関係なく行われる
	drop(slot_map);
}

// `num_elems`(len/is_empty)がinsert/removeを経ても
// 現在生きている要素数を正しく追跡できているか
// (inner_len = slots.len()とは異なる概念であることの確認でもある)
#[test]
fn len_and_is_empty_track_live_element_count() {
	let mut m = UnsafeSlotMap::new_for::<u32>();
	assert!(m.is_empty());
	assert_eq!(m.len(), 0);

	let idx1 = unsafe { m.insert_for(1u32) };
	let idx2 = unsafe { m.insert_for(2u32) };
	assert_eq!(m.len(), 2);
	assert!(!m.is_empty());

	let _ = unsafe { m.remove_for::<u32>(idx1) };
	assert_eq!(m.len(), 1);

	let _ = unsafe { m.remove_for::<u32>(idx2) };
	assert_eq!(m.len(), 0);
	assert!(m.is_empty());
}

// 空きリストはEMPTY(next_free)のリンクリストなので、
// 挿入順とは異なる順序・複数個をremoveした場合でも
// (1) 再利用されるインデックスがremoveした順序の逆順になること
// (2) removeしていない他の要素の値が一切壊れていないこと
// を確認する
// free_headの連結が壊れていれば、ここで再利用順序かデータ破壊で検出できる
#[test]
fn free_list_reuses_indices_in_lifo_order_without_corrupting_other_elements() {
	let mut m = UnsafeSlotMap::new_for::<u32>();
	let idxs: Vec<usize> = (0..6u32).map(|v| unsafe { m.insert_for(v * 10) }).collect();

	// 挿入順ではなく 2, 4, 1 の順で remove する。
	let removed_2 = unsafe { m.remove_for::<u32>(idxs[2]) };
	let removed_4 = unsafe { m.remove_for::<u32>(idxs[4]) };
	let removed_1 = unsafe { m.remove_for::<u32>(idxs[1]) };
	assert_eq!((removed_2, removed_4, removed_1), (20, 40, 10));
	assert_eq!(m.len(), 3);

	// 空きリストは remove の"逆順"(LIFO)で消費されるはず: 1 → 4 → 2
	let re1 = unsafe { m.insert_for(100u32) };
	let re2 = unsafe { m.insert_for(200u32) };
	let re3 = unsafe { m.insert_for(300u32) };
	assert_eq!(re1, idxs[1]);
	assert_eq!(re2, idxs[4]);
	assert_eq!(re3, idxs[2]);

	// 一度も remove していない要素 (0, 3, 5) の値が壊れていないことを確認。
	assert_eq!(*unsafe { m.get_for::<u32>(idxs[0]) }, 0);
	assert_eq!(*unsafe { m.get_for::<u32>(idxs[3]) }, 30);
	assert_eq!(*unsafe { m.get_for::<u32>(idxs[5]) }, 50);
	assert_eq!(m.len(), 6);
}

// `grow_for`は複数回の`realloc`を経ることになるが、その際に
// 古いlayoutのサイズ計算を誤って一部データを取りこぼす/破壊するような回帰が起きていないかを、十分な要素数(複数回の再確保が起こる規模)で全件チェックする
// 特にcapacity_heuristicの初期値(4)から10000を超えるまで倍々に成長する過程を通す
#[test]
fn regrowth_preserves_previously_inserted_values() {
	let mut m = UnsafeSlotMap::new_for::<u64>();
	const N: u64 = 10_000;

	for i in 0..N {
		let idx = unsafe { m.insert_for(i) };
		assert_eq!(idx, i as usize);
	}

	for i in 0..N {
		assert_eq!(
			*unsafe { m.get_for::<u64>(i as usize) },
			i,
			"grow(realloc)後にindex{i}の値が破壊されている"
		);
	}
}

// `u32`/`u64`のような素朴な型だけでは、`Layout::array::<T>`のアラインメント計算ミスは表面化しにくいので
// アラインメント要求が大きい型(ここでは64バイト境界)を使い、各要素が実際にそのアラインメントのアドレスに配置されていることをポインタ演算で直接確認する
#[test]
fn respects_large_alignment_requirements() {
	#[repr(align(64))]
	#[derive(Debug, Clone, Copy, PartialEq)]
	struct Aligned64 {
		value: u64,
	}

	let mut m = UnsafeSlotMap::new_for::<Aligned64>();
	let idxs: Vec<usize> =
		(0..50u64).map(|i| unsafe { m.insert_for(Aligned64 { value: i }) }).collect();

	for (i, idx) in idxs.iter().enumerate() {
		let got = unsafe { m.get_for::<Aligned64>(*idx) };
		assert_eq!(got.value, i as u64);

		let addr = got as *const Aligned64 as usize;
		assert_eq!(addr % 64, 0, "index{idx}の要素がアラインメント64に沿っていない");
	}
}

// `push_mut_for`はpushした要素への可変参照を返す
// 返された参照が本当にpushした要素そのものを指しており、それを介した書き換えが
// 後続の読み出しにも反映されることを確認する
#[test]
fn push_mut_for_returns_reference_to_the_pushed_element() {
	let mut v = UnsafeVec::new_for::<u32>();
	let r = unsafe { v.push_mut_for(10u32) };
	*r += 5;

	assert_eq!(unsafe { *v.get_for::<u32>(0) }, 15);
}

// `get_mut_for`によるインプレース変更が、実際にバッファ上の値を
// 書き換えていること(単に一時的なコピーを返しているだけではないこと)を確認する
#[test]
fn get_mut_for_allows_in_place_mutation() {
	let mut v = UnsafeVec::new_for::<u32>();
	unsafe { v.push_for(1u32) };
	*unsafe { v.get_mut_for::<u32>(0) } += 41;

	assert_eq!(unsafe { *v.get_for::<u32>(0) }, 42);
}

// `debug_assert_type`が実際に型の取り違え(サイズ不一致)を検出して
// panicすることを確認する
// これは安全網が机上の議論で終わっておらず、実際に発火することを証明するテストであり、debug_assertionsが
// 無効なreleaseビルドでは検査自体が存在しないため実行しない
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "型のサイズが構築時と一致しません")]
fn type_mismatch_is_caught_by_debug_assert() {
	let mut v = UnsafeVec::new_for::<u32>();
	unsafe { v.push_for(42u32) };

	// 誤って構築時とは異なる型として読み出す(コンパイル時には検出できない誤用)
	let _ = unsafe { v.get_for::<u64>(0) };
}

// `remove_for`の境界チェックが実際に機能していることを確認する。
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "範囲外")]
fn out_of_bounds_access_is_caught_by_debug_assert() {
	let v = UnsafeVec::new_for::<u32>();
	// 何も push していないので、index 0 は既に範囲外。
	let _ = unsafe { v.get_for::<u32>(0) };
}
