use std::{marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use crate::widget::{Widget, WidgetHandle, WidgetType};

pub struct Reservation<T: Widget + 'static> {
	ptr: NonNull<T>,
	ty: &'static WidgetType,
	_phantom: PhantomData<T>,
}

impl<T: Widget + 'static> Reservation<T> {
	#[inline(always)]
	#[must_use]
	pub(crate) unsafe fn from_raw(ptr: NonNull<T>, ty: &'static WidgetType) -> Self {
		Self { ptr, ty, _phantom: PhantomData }
	}

	#[inline]
	#[must_use]
	pub(crate) fn as_ptr(&self) -> NonNull<T> {
		self.ptr
	}

	#[inline]
	#[must_use]
	pub(crate) fn write(self, widget: T) -> WidgetHandle {
		let this = ManuallyDrop::new(self);
		unsafe { this.ptr.as_ptr().write(widget) };

		unsafe { WidgetHandle::from_raw(this.ptr.cast(), this.ty) }
	}
}

impl<T: Widget + 'static> Drop for Reservation<T> {
	fn drop(&mut self) {
		unsafe { self.ty.pool().release::<T>(self.ptr.as_ptr()) };
	}
}
