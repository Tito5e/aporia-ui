use std::num::NonZeroUsize;

use crate::storage::unsafe_vec::UnsafeVec;

enum Slot<T> {
	Occupied(T),
	Empty(usize),
	Reserved,
}

impl<T> Slot<T> {
	#[inline]
	const fn occupied(&self) -> bool {
		matches!(self, Slot::Occupied(_))
	}

	#[inline]
	const fn reserved(&self) -> bool {
		matches!(self, Slot::Reserved)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Key(usize);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ReserveKey(usize);

#[derive(Debug)]
pub(crate) struct UnsafeSlotMap {
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
	pub unsafe fn insert_for<T>(&mut self, value: T) -> Key {
		let slot_idx = self.free_head;
		let inner_len = self.slots.len();

		if self.num_elems < inner_len {
			let slot = unsafe { self.slots.get_mut_for::<Slot<T>>(slot_idx) };
			let next_free = match slot {
				Slot::Empty(next_free) => *next_free,
				Slot::Occupied(_) => unreachable!(),
				Slot::Reserved => unreachable!(),
			};
			*slot = Slot::Occupied(value);
			self.free_head = next_free;
		} else {
			unsafe { self.slots.push_for(Slot::Occupied(value)) };
			self.free_head = self.num_elems + 1;
		}

		self.num_elems += 1;

		Key(slot_idx)
	}

	#[inline]
	#[must_use]
	pub unsafe fn reserve_for<T>(&mut self) -> ReserveKey {
		let slot_idx = self.free_head;
		let inner_len = self.slots.len();

		if self.num_elems < inner_len {
			let slot = unsafe { self.slots.get_mut_for::<Slot<T>>(slot_idx) };
			let next_free = match slot {
				Slot::Empty(next_free) => *next_free,
				Slot::Occupied(_) => unreachable!(),
				Slot::Reserved => unreachable!(),
			};
			*slot = Slot::Reserved;
			self.free_head = next_free;
		} else {
			unsafe { self.slots.push_for::<Slot<T>>(Slot::Reserved) };
			self.free_head = self.num_elems + 1;
		}

		self.num_elems += 1;

		ReserveKey(slot_idx)
	}

	#[inline]
	pub unsafe fn write_reserved_for<T>(&mut self, idx: ReserveKey, value: T) -> Key {
		let slot = unsafe { self.slots.get_mut_for::<Slot<T>>(idx.0) };

		match slot {
			Slot::Empty(_) => unreachable!(),
			Slot::Occupied(_) => unreachable!(),
			Slot::Reserved => *slot = Slot::Occupied(value),
		}

		Key(idx.0)
	}

	#[inline]
	pub unsafe fn cancel_reservation_for<T>(&mut self, idx: ReserveKey) {
		let slot = unsafe { self.slots.get_mut_for::<Slot<T>>(idx.0) };

		match slot {
			Slot::Reserved => {
				*slot = Slot::Empty(self.free_head);
				self.free_head = idx.0;
				self.num_elems -= 1;
			}
			Slot::Empty(_) => unreachable!(),
			Slot::Occupied(_) => unreachable!(),
		}
	}

	/// # Safety
	/// `idx`は現在occupiedなスロットのインデックスである必要がある
	/// `T`はこのインスタンスの構築時に使われた型と一致していなければならない
	#[inline]
	#[must_use]
	pub unsafe fn get_for<T>(&self, idx: Key) -> &T {
		let slot = unsafe { self.slots.get_for::<Slot<T>>(idx.0) };

		match slot {
			Slot::Empty(_) => unreachable!(),
			Slot::Reserved => unreachable!("tempo"),
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
	pub unsafe fn remove_for<T>(&mut self, idx: Key) -> T {
		let slot = unsafe { self.slots.get_mut_for::<Slot<T>>(idx.0) };
		let slot_data = std::mem::replace(slot, Slot::Empty(self.free_head));
		self.free_head = idx.0;
		self.num_elems -= 1;

		match slot_data {
			Slot::Empty(_) => unreachable!(),
			Slot::Occupied(value) => value,
			Slot::Reserved => unreachable!(),
		}
	}
}

#[test]
pub fn unsafe_slot_map() {
	let mut slot_map = UnsafeSlotMap::new_for::<u32>();
	let idx1 = unsafe { slot_map.insert_for::<u32>(5) };
	let idx2 = unsafe { slot_map.insert_for::<u32>(8) };
	assert!(idx1.0 == 0);
	assert!(idx2.0 == 1);

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

	let _ = unsafe { slot_map.remove_for::<Rc<()>>(idx1) };
	let _ = unsafe { slot_map.remove_for::<Rc<()>>(idx2) };
	assert_eq!(Rc::strong_count(&counter), 1);

	drop(slot_map);
}

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

#[test]
fn free_list_reuses_indices_in_lifo_order_without_corrupting_other_elements() {
	let mut m = UnsafeSlotMap::new_for::<u32>();
	let idxs: Vec<Key> = (0..6u32).map(|v| unsafe { m.insert_for(v * 10) }).collect();

	let removed_2 = unsafe { m.remove_for::<u32>(idxs[2]) };
	let removed_4 = unsafe { m.remove_for::<u32>(idxs[4]) };
	let removed_1 = unsafe { m.remove_for::<u32>(idxs[1]) };
	assert_eq!((removed_2, removed_4, removed_1), (20, 40, 10));
	assert_eq!(m.len(), 3);

	let re1 = unsafe { m.insert_for(100u32) };
	let re2 = unsafe { m.insert_for(200u32) };
	let re3 = unsafe { m.insert_for(300u32) };
	assert_eq!(re1, idxs[1]);
	assert_eq!(re2, idxs[4]);
	assert_eq!(re3, idxs[2]);

	assert_eq!(*unsafe { m.get_for::<u32>(idxs[0]) }, 0);
	assert_eq!(*unsafe { m.get_for::<u32>(idxs[3]) }, 30);
	assert_eq!(*unsafe { m.get_for::<u32>(idxs[5]) }, 50);
	assert_eq!(m.len(), 6);
}

#[test]
fn regrowth_preserves_previously_inserted_values() {
	let mut m = UnsafeSlotMap::new_for::<u64>();
	const N: u64 = 10_000;

	for i in 0..N {
		let idx = unsafe { m.insert_for(i) };
		assert_eq!(idx.0, i as usize);
	}

	for i in 0..N {
		assert_eq!(
			*unsafe { m.get_for::<u64>(Key(i as usize)) },
			i,
			"grow(realloc)後にindex{i}の値が破壊されている"
		);
	}
}

#[test]
fn respects_large_alignment_requirements() {
	#[repr(align(64))]
	#[derive(Debug, Clone, Copy, PartialEq)]
	struct Aligned64 {
		value: u64,
	}

	let mut m = UnsafeSlotMap::new_for::<Aligned64>();
	let idxs: Vec<Key> =
		(0..50u64).map(|i| unsafe { m.insert_for(Aligned64 { value: i }) }).collect();

	for (i, idx) in idxs.iter().enumerate() {
		let got = unsafe { m.get_for::<Aligned64>(*idx) };
		assert_eq!(got.value, i as u64);

		let addr = got as *const Aligned64 as usize;
		assert_eq!(addr % 64, 0, "index{}の要素がアラインメント64に沿っていない", idx.0);
	}
}

#[test]
fn push_mut_for_returns_reference_to_the_pushed_element() {
	let mut v = UnsafeVec::new_for::<u32>();
	let r = unsafe { v.push_mut_for(10u32) };
	*r += 5;

	assert_eq!(unsafe { *v.get_for::<u32>(0) }, 15);
}

#[test]
fn get_mut_for_allows_in_place_mutation() {
	let mut v = UnsafeVec::new_for::<u32>();
	unsafe { v.push_for(1u32) };
	*unsafe { v.get_mut_for::<u32>(0) } += 41;

	assert_eq!(unsafe { *v.get_for::<u32>(0) }, 42);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "型のサイズが構築時と一致しません")]
fn type_mismatch_is_caught_by_debug_assert() {
	let mut v = UnsafeVec::new_for::<u32>();
	unsafe { v.push_for(42u32) };

	let _ = unsafe { v.get_for::<u64>(0) };
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "範囲外")]
fn out_of_bounds_access_is_caught_by_debug_assert() {
	let v = UnsafeVec::new_for::<u32>();

	let _ = unsafe { v.get_for::<u32>(0) };
}

#[test]
fn reserve_then_write_supports_self_referential_index() {
	struct Node {
		self_idx: ReserveKey,
		value: u32,
	}

	let mut m = UnsafeSlotMap::new_for::<Node>();

	let idx = unsafe { m.reserve_for::<Node>() };

	let node = Node { self_idx: idx, value: 42 };
	let idx = unsafe { m.write_reserved_for(idx, node) };

	let got = unsafe { m.get_for::<Node>(idx) };
	assert_eq!(got.self_idx.0, idx.0);
	assert_eq!(got.value, 42);
}

#[test]
fn cancel_reservation_returns_index_to_free_list() {
	let mut m = UnsafeSlotMap::new_for::<u32>();
	let idx = unsafe { m.reserve_for::<u32>() };
	assert_eq!(m.len(), 1);

	unsafe { m.cancel_reservation_for::<u32>(idx) };
	assert_eq!(m.len(), 0);

	let idx2 = unsafe { m.insert_for(7u32) };
	assert_eq!(idx2.0, idx.0);
	assert_eq!(*unsafe { m.get_for::<u32>(idx2) }, 7);
}

#[test]
fn reserve_for_and_insert_for_share_the_free_list_correctly() {
	let mut m = UnsafeSlotMap::new_for::<u32>();

	let idx_a = unsafe { m.insert_for(1u32) };
	let idx_b = unsafe { m.reserve_for::<u32>() };
	let idx_c = unsafe { m.insert_for(3u32) };
	assert_eq!((idx_a.0, idx_b.0, idx_c.0), (0, 1, 2));

	let removed_a = unsafe { m.remove_for::<u32>(idx_a) };
	assert_eq!(removed_a, 1);
	let idx_d = unsafe { m.reserve_for::<u32>() };
	assert_eq!(idx_d.0, idx_a.0, "removeで空いたスロットがLIFOで再利用される");

	assert_eq!(*unsafe { m.get_for::<u32>(idx_c) }, 3);

	let idx_b = unsafe { m.write_reserved_for(idx_b, 20) };
	let idx_d = unsafe { m.write_reserved_for(idx_d, 40) };
	assert_eq!(*unsafe { m.get_for::<u32>(idx_b) }, 20);
	assert_eq!(*unsafe { m.get_for::<u32>(idx_d) }, 40);
}
