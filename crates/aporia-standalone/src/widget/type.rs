#[cfg(debug_assertions)]
use std::any::TypeId;
use std::{
	cell::{RefCell, UnsafeCell},
	collections::HashMap,
	ffi::c_void,
	ptr::NonNull,
};

use aporia_core::geometry::{Constraint, Size};
use unsafe_pool::UnsafePool;

use crate::widget::Widget;

#[derive(Debug)]
pub(crate) struct WidgetType {
	pool: UnsafeCell<UnsafePool>,
	layout: unsafe fn(NonNull<c_void>, Constraint) -> Size,
	drop: unsafe fn(&'static WidgetType, NonNull<c_void>),
	#[cfg(debug_assertions)]
	pub(crate) type_id: TypeId,
}

thread_local! {
	static TYPES: RefCell<HashMap<TypeId, &'static WidgetType>> = RefCell::new(HashMap::new());
}

impl WidgetType {
	pub(crate) fn of<T: Widget + 'static>() -> &'static WidgetType {
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
	#[allow(clippy::mut_from_ref)]
	pub(crate) unsafe fn pool(&'static self) -> &'static mut UnsafePool {
		unsafe { &mut *self.pool.get() }
	}

	#[inline]
	pub(crate) unsafe fn layout(&self, ptr: NonNull<c_void>, constraint: Constraint) -> Size {
		unsafe { (self.layout)(ptr, constraint) }
	}

	#[inline]
	pub(crate) unsafe fn drop(&self, r#type: &'static WidgetType, ptr: NonNull<c_void>) {
		unsafe { (self.drop)(r#type, ptr) }
	}
}
