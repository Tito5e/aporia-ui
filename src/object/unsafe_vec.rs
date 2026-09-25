use std::{
	alloc::{Layout, alloc},
	ffi::c_void,
	num::NonZeroUsize,
	ptr::null_mut,
};

/// 型消去して要素を確保できるUnsafeな配列
/// 要素を削除することはできず、一括でDropすることもできない
/// 保存する要素は利用者側で責任を持ってDropを管理する必要がある
#[derive(Debug)]
pub struct UnsafeVec {
	ptr: *mut c_void,
	cap: usize,
	len: usize,
	size: usize,
	align: usize,
}

impl UnsafeVec {
	#[inline]
	#[must_use]
	pub(crate) const fn new_for<T>() -> Self {
		Self { ptr: null_mut(), cap: 0, len: 0, size: size_of::<T>(), align: align_of::<T>() }
	}

	#[inline]
	#[must_use]
	pub unsafe fn with_capacity_for<T>(capacity: NonZeroUsize) -> Self {
		let layout = unsafe { Layout::from_size_align_unchecked(size_of::<T>(), align_of::<T>()) };
		let array_layout = layout.repeat_packed(capacity.get()).expect("Layout Error");
		let ptr = unsafe { alloc(array_layout) } as *mut c_void;

		Self { ptr, cap: capacity.get(), len: 0, size: size_of::<T>(), align: align_of::<T>() }
	}

	#[inline]
	pub unsafe fn push_for<T>(&mut self, value: T) {
		let _ = unsafe { self.push_mut_for(value) };
	}

	#[inline]
	// TODO: Comment
	#[must_use]
	pub unsafe fn push_mut_for<T>(&mut self, value: T) -> &mut T {
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

	#[inline]
	#[must_use]
	pub unsafe fn get_for<T>(&self, index: usize) -> &T {
		let end = unsafe { self.get_ptr_for::<T>(index) };

		unsafe { &*end }
	}

	#[inline]
	#[must_use]
	pub unsafe fn get_mut_for<T>(&mut self, index: usize) -> &mut T {
		let end = unsafe { self.get_mut_ptr_for::<T>(index) };

		unsafe { &mut *end }
	}

	#[inline]
	#[must_use]
	pub unsafe fn get_ptr_for<T>(&self, index: usize) -> *const T {
		let end = unsafe { self.as_ptr_for::<T>().add(index) };

		end
	}

	#[inline]
	#[must_use]
	pub unsafe fn get_mut_ptr_for<T>(&mut self, index: usize) -> *mut T {
		let end = unsafe { self.as_mut_ptr_for::<T>().add(index) };

		end
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

		let layout = unsafe { Layout::from_size_align_unchecked(size_of::<T>(), align_of::<T>()) };
		let new_array_layout = layout.repeat_packed(optimal_cap).expect("Layout Error");

		let ptr = unsafe {
			if self.cap == 0 {
				std::alloc::alloc(new_array_layout)
			} else {
				let array_layout = layout.repeat_packed(self.cap).expect("Layout Error");
				std::alloc::realloc(self.ptr as *mut u8, array_layout, new_array_layout.size())
			}
		};

		self.ptr = ptr as *mut c_void;
		self.cap = optimal_cap;
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

/// Rustのstdから持ってきたコード
/// 要素のサイズが1バイトのときは8バイトになるように詰め、1kB以下のときは4つは確保する、それ以上の大きさの場合は最低1個とする
const fn capacity_heuristic(element_size: usize) -> usize {
	if element_size == 1 {
		8
	} else if element_size <= 1024 {
		4
	} else {
		1
	}
}
