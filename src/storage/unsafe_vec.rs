use std::{
	alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc},
	ffi::c_void,
	num::NonZeroUsize,
	ptr::null_mut,
};

#[derive(Debug)]
pub struct UnsafeVec {
	ptr: *mut c_void,
	cap: usize,
	len: usize,
	dealloc_fn: unsafe fn(*mut c_void, usize),

	#[cfg(debug_assertions)]
	size: usize,
	#[cfg(debug_assertions)]
	align: usize,
}

unsafe fn dealloc_buffer<T>(ptr: *mut c_void, cap: usize) {
	if cap == 0 {
		return;
	}

	let layout = Layout::array::<T>(cap).expect("Layout Error");
	unsafe { dealloc(ptr as *mut u8, layout) };
}

impl UnsafeVec {
	#[inline]
	#[must_use]
	pub(crate) const fn new_for<T>() -> Self {
		Self {
			ptr: null_mut(),
			cap: 0,
			len: 0,
			dealloc_fn: dealloc_buffer::<T>,
			#[cfg(debug_assertions)]
			size: size_of::<T>(),
			#[cfg(debug_assertions)]
			align: align_of::<T>(),
		}
	}

	#[inline]
	#[must_use]
	pub unsafe fn with_capacity_for<T>(capacity: NonZeroUsize) -> Self {
		if size_of::<T>() == 0 {
			return Self::new_for::<T>();
		}

		let capacity = capacity.get();
		let array_layout = Layout::array::<T>(capacity).expect("Layout Error");

		let raw = unsafe { alloc(array_layout) };
		if raw.is_null() {
			handle_alloc_error(array_layout);
		}

		Self {
			ptr: raw as *mut c_void,
			cap: capacity,
			len: 0,
			dealloc_fn: dealloc_buffer::<T>,
			#[cfg(debug_assertions)]
			size: size_of::<T>(),
			#[cfg(debug_assertions)]
			align: align_of::<T>(),
		}
	}

	#[inline]
	pub unsafe fn push_for<T>(&mut self, value: T) {
		let _ = unsafe { self.push_mut_for(value) };
	}

	#[inline]
	pub unsafe fn push_mut_for<T>(&mut self, value: T) -> &mut T {
		self.debug_assert_type::<T>();

		let len = self.len;
		if len == self.cap {
			unsafe { self.grow_one_for::<T>() };
		}

		unsafe {
			let end = self.as_mut_ptr_for::<T>().add(len);
			std::ptr::write(end, value);
			self.len = len + 1;
			&mut *end
		}
	}

	/// # Safety
	/// `index < self.len()`であること
	/// `T`が構築時の型と一致していること
	#[inline]
	#[must_use]
	pub unsafe fn get_for<T>(&self, index: usize) -> &T {
		let ptr = unsafe { self.get_ptr_for::<T>(index) };
		unsafe { &*ptr }
	}

	/// # Safety
	/// `index < self.len()`であること
	/// `T`が構築時の型と一致していること
	#[inline]
	#[must_use]
	pub unsafe fn get_mut_for<T>(&mut self, index: usize) -> &mut T {
		let ptr = unsafe { self.get_mut_ptr_for::<T>(index) };
		unsafe { &mut *ptr }
	}

	/// # Safety
	/// `index < self.len()`であること
	/// `T`が構築時の型と一致していること
	#[inline]
	#[must_use]
	pub unsafe fn get_ptr_for<T>(&self, index: usize) -> *const T {
		self.debug_assert_type::<T>();
		debug_assert!(
			index < self.len,
			"UnsafeVec: index {index} out of bounds (len={}).",
			self.len
		);

		unsafe { self.as_ptr_for::<T>().add(index) }
	}

	/// # Safety
	/// `index < self.len()`であること
	/// `T`が構築時の型と一致していること
	#[inline]
	#[must_use]
	pub unsafe fn get_mut_ptr_for<T>(&mut self, index: usize) -> *mut T {
		self.debug_assert_type::<T>();
		debug_assert!(
			index < self.len,
			"UnsafeVec: index {index} out of bounds (len={}).",
			self.len
		);

		unsafe { self.as_mut_ptr_for::<T>().add(index) }
	}

	#[inline]
	unsafe fn grow_one_for<T>(&mut self) {
		unsafe { self.grow_for::<T>(1) }
	}

	unsafe fn grow_for<T>(&mut self, additional: usize) {
		debug_assert!(additional > 0);

		if size_of::<T>() == 0 {
			return;
		}

		let required_cap = self.len.checked_add(additional).expect("Capacity Overflow");

		let optimal_cap = std::cmp::max(self.cap.saturating_mul(2), required_cap);
		let optimal_cap = std::cmp::max(capacity_heuristic(size_of::<T>()), optimal_cap);

		let new_array_layout = Layout::array::<T>(optimal_cap).expect("Layout Error");

		let ptr = unsafe {
			if self.cap == 0 {
				alloc(new_array_layout)
			} else {
				let array_layout = Layout::array::<T>(self.cap).expect("Layout Error");
				realloc(self.ptr as *mut u8, array_layout, new_array_layout.size())
			}
		};

		if ptr.is_null() {
			handle_alloc_error(new_array_layout);
		}

		self.ptr = ptr as *mut c_void;
		self.cap = optimal_cap;
	}

	#[cfg(debug_assertions)]
	#[inline]
	fn debug_assert_type<T>(&self) {
		debug_assert_eq!(
			self.size,
			size_of::<T>(),
			"UnsafeVec: type size does not match the size specified at initialization."
		);
		debug_assert_eq!(
			self.align,
			align_of::<T>(),
			"UnsafeVec: type alignment does not match the alignment specified at initialization."
		);
	}

	#[cfg(not(debug_assertions))]
	#[inline(always)]
	fn debug_assert_type<T>(&self) {
		// Nothing
	}

	#[inline]
	pub const fn capacity(&self) -> usize {
		self.cap
	}

	#[inline]
	pub const fn len(&self) -> usize {
		self.len
	}

	#[inline]
	pub const unsafe fn as_ptr_for<T>(&self) -> *const T {
		self.ptr as *const T
	}

	#[inline]
	pub const unsafe fn as_mut_ptr_for<T>(&mut self) -> *mut T {
		self.ptr as *mut T
	}
}

impl Drop for UnsafeVec {
	fn drop(&mut self) {
		unsafe { (self.dealloc_fn)(self.ptr, self.cap) };
	}
}

const fn capacity_heuristic(element_size: usize) -> usize {
	if element_size == 1 {
		8
	} else if element_size <= 1024 {
		4
	} else {
		1
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
#[should_panic(
	expected = "UnsafeVec: type size does not match the size specified at initialization."
)]
fn type_mismatch_is_caught_by_debug_assert() {
	let mut v = UnsafeVec::new_for::<u32>();
	unsafe { v.push_for(42u32) };

	let _ = unsafe { v.get_for::<u64>(0) };
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "out of bounds")]
fn out_of_bounds_access_is_caught_by_debug_assert() {
	let v = UnsafeVec::new_for::<u32>();

	let _ = unsafe { v.get_for::<u32>(0) };
}
