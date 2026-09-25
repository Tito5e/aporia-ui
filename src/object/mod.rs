// TODO: implement custom vec
pub mod unsafe_vec;

use std::num::NonZeroUsize;

use crate::object::unsafe_vec::UnsafeVec;

enum Slot<T> {
	OCCUPIED(T),
	EMPTY(usize),
}

impl<T> Slot<T> {
	#[inline(always)]
	pub fn occupied(&self) -> bool {
		match self {
			Slot::OCCUPIED(_) => true,
			Slot::EMPTY(_) => false,
		}
	}
}

#[derive(Debug)]
pub struct ErasedSlotMap {
	slots: UnsafeVec,
	free_head: usize,
	num_elems: usize,
}

impl ErasedSlotMap {
	pub const fn new_for<T>() -> Self {
		let slots = UnsafeVec::new_for::<Slot<T>>();

		Self { slots, free_head: 0, num_elems: 0 }
	}

	pub fn with_capacity_for<T>(capacity: NonZeroUsize) -> Self {
		let slots = unsafe { UnsafeVec::with_capacity_for::<Slot<T>>(capacity) };

		Self { slots, free_head: 0, num_elems: 0 }
	}

	pub fn len(&self) -> usize {
		self.num_elems as usize
	}

	pub fn is_empty(&self) -> bool {
		self.num_elems == 0
	}

	pub unsafe fn contains_for<T>(&self, idx: usize) -> bool {
		let slot = unsafe { self.slots.get_for::<Slot<T>>(idx) };
		slot.occupied()
	}

	#[inline]
	#[must_use]
	pub unsafe fn insert_for<T>(&mut self, value: T) -> usize {
		let slot_idx = self.free_head as usize;
		let inner_len = self.slots.len();
		if self.num_elems < inner_len {
			let slot = unsafe { self.slots.get_mut_ptr_for::<Slot<T>>(slot_idx) };
			let slot_data = unsafe { &mut *slot };
			let next_free = match slot_data {
				Slot::EMPTY(next_free) => *next_free,
				Slot::OCCUPIED(_) => unreachable!(""),
			};
			*slot_data = Slot::OCCUPIED(value);
			self.free_head = next_free;
			self.num_elems += 1;

			slot_idx
		} else {
			unsafe { self.slots.push_for::<Slot<T>>(Slot::OCCUPIED(value)) };
			self.free_head = self.num_elems + 1;
			self.num_elems += 1;

			slot_idx
		}
	}

	#[inline]
	#[must_use]
	pub unsafe fn get_for<T>(&self, idx: usize) -> &T {
		let slot = unsafe { self.slots.get_for::<Slot<T>>(idx) };

		match slot {
			Slot::EMPTY(_) => unreachable!(),
			Slot::OCCUPIED(value) => value,
		}
	}

	#[inline]
	#[must_use]
	pub unsafe fn remove_for<T>(&mut self, idx: usize) -> T {
		if self.slots.capacity() < idx {
			panic!("")
		}
		let slot = unsafe { self.slots.get_mut_for::<Slot<T>>(idx) };
		let slot_data = std::mem::replace(slot, Slot::EMPTY(self.free_head));
		self.free_head = idx;
		self.num_elems -= 1;

		match slot_data {
			Slot::EMPTY(_) => unreachable!(),
			Slot::OCCUPIED(value) => value,
		}
	}
}

#[test]
pub fn erased_slot_map() {
	let mut slot_map = ErasedSlotMap::new_for::<u32>();
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
