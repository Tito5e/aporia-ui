#[cfg(debug_assertions)]
use std::any::TypeId;
use std::{
	cell::{RefCell, UnsafeCell},
	collections::HashMap,
	ffi::c_void,
	marker::PhantomData,
	mem::ManuallyDrop,
	ptr::NonNull,
};

use crate::{
	core::geometry::{Constraint, Size},
	storage::UnsafePool,
	widget::Widget,
};

pub(crate) struct WidgetType {
	pool: UnsafeCell<UnsafePool>,
	layout: unsafe fn(NonNull<c_void>, Constraint) -> Size,
	drop: unsafe fn(&'static WidgetType, NonNull<c_void>),
	#[cfg(debug_assertions)]
	type_id: TypeId,
}

thread_local! {
	static TYPES: RefCell<HashMap<TypeId, &'static WidgetType>> = RefCell::new(HashMap::new());
}

impl WidgetType {
	fn of<T: Widget + 'static>() -> &'static WidgetType {
		TYPES.with(|types| {
			*types.borrow_mut().entry(TypeId::of::<T>()).or_insert_with(|| {
				Box::leak(Box::new(WidgetType {
					pool: UnsafeCell::new(UnsafePool::new::<T>()),
					layout: |ptr, constraint| {
						let widget = unsafe { &mut *ptr.as_ptr().cast::<T>() };
						widget.layout(constraint)
					},
					drop: |ty, ptr| {
						let value = unsafe { (*ty.pool.get()).remove::<T>(ptr.as_ptr().cast()) };

						drop(value);
					},
					#[cfg(debug_assertions)]
					type_id: TypeId::of::<T>(),
				}))
			})
		})
	}

	#[inline]
	unsafe fn pool(&'static self) -> &'static mut UnsafePool {
		unsafe { &mut *self.pool.get() }
	}
}

pub struct WidgetHandle {
	ptr: NonNull<c_void>,
	ty: &'static WidgetType,
}

impl WidgetHandle {
	pub(crate) fn new<T: Widget + 'static>(widget: T) -> Self {
		let ty = WidgetType::of::<T>();
		let ptr = unsafe { ty.pool().insert(widget) };

		Self { ptr: unsafe { NonNull::new_unchecked(ptr).cast() }, ty }
	}

	pub(crate) fn reserve<T: Widget + 'static>() -> Reservation<T> {
		let ty = WidgetType::of::<T>();
		let ptr = unsafe { ty.pool().reserve::<T>() };

		Reservation { ptr: unsafe { NonNull::new_unchecked(ptr) }, ty, _phantom: PhantomData }
	}

	#[inline]
	pub fn layout(&mut self, constraint: Constraint) -> Size {
		unsafe { (self.ty.layout)(self.ptr, constraint) }
	}

	#[inline]
	pub(crate) fn as_ptr(&self) -> NonNull<c_void> {
		self.ptr
	}

	#[inline]
	pub(crate) unsafe fn get<T: Widget + 'static>(&self) -> &T {
		self.debug_assert_type::<T>();

		unsafe { &*self.ptr.as_ptr().cast::<T>() }
	}

	#[inline]
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
		unsafe { (self.ty.drop)(self.ty, self.ptr) }
	}
}

pub(crate) struct Reservation<T: Widget + 'static> {
	ptr: NonNull<T>,
	ty: &'static WidgetType,
	_phantom: PhantomData<T>,
}

impl<T: Widget + 'static> Reservation<T> {
	#[inline]
	pub(crate) fn as_ptr(&self) -> NonNull<c_void> {
		self.ptr.cast()
	}

	pub(crate) fn write(self, widget: T) -> WidgetHandle {
		let this = ManuallyDrop::new(self);
		unsafe { this.ptr.as_ptr().write(widget) };

		WidgetHandle { ptr: this.ptr.cast(), ty: this.ty }
	}
}

impl<T: Widget + 'static> Drop for Reservation<T> {
	fn drop(&mut self) {
		unsafe { self.ty.pool().release::<T>(self.ptr.as_ptr()) };
	}
}
