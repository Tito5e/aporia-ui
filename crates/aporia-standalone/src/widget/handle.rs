use std::{any::TypeId, ffi::c_void, ptr::NonNull};

use aporia_core::geometry::{Constraint, Size};

use crate::widget::{Reservation, Widget, WidgetType};

pub struct WidgetHandle {
	ptr: NonNull<c_void>,
	ty: &'static WidgetType,
}

impl WidgetHandle {
	#[inline]
	#[must_use]
	pub(crate) fn new<T: Widget + 'static>(widget: T) -> Self {
		let ty = WidgetType::of::<T>();
		let ptr = unsafe { ty.pool().insert(widget) };

		Self { ptr: unsafe { NonNull::new_unchecked(ptr).cast() }, ty }
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn from_raw(ptr: NonNull<c_void>, ty: &'static WidgetType) -> Self {
		Self { ptr, ty }
	}

	#[inline]
	#[must_use]
	pub(crate) fn reserve<T: Widget + 'static>() -> Reservation<T> {
		let ty = WidgetType::of::<T>();
		let ptr = unsafe { ty.pool().reserve::<T>() };

		unsafe { Reservation::from_raw(unsafe { NonNull::new_unchecked(ptr) }, ty) }
	}

	#[inline]
	#[must_use]
	pub fn layout(&mut self, constraint: Constraint) -> Size {
		unsafe { self.ty.layout(self.ptr, constraint) }
	}

	#[inline]
	#[must_use]
	pub(crate) fn as_ptr(&self) -> NonNull<c_void> {
		self.ptr
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn get<T: Widget + 'static>(&self) -> &T {
		self.debug_assert_type::<T>();

		unsafe { &*self.ptr.as_ptr().cast::<T>() }
	}

	#[inline]
	#[must_use]
	pub(crate) unsafe fn get_mut<T: Widget + 'static>(&mut self) -> &mut T {
		self.debug_assert_type::<T>();

		unsafe { &mut *self.ptr.as_ptr().cast::<T>() }
	}

	#[inline(always)]
	fn debug_assert_type<T: 'static>(&self) {
		#[cfg(debug_assertions)]
		debug_assert_eq!(self.ty.type_id, TypeId::of::<T>(), "Type Mismatch.")
	}
}

impl Drop for WidgetHandle {
	fn drop(&mut self) {
		unsafe { self.ty.drop(self.ty, self.ptr) }
	}
}
