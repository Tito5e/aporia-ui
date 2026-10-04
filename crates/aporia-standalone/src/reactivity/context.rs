use std::{any::TypeId, collections::HashMap, ptr::NonNull};
use unsafe_pool::UnsafePool;

pub struct Context {
	pools: HashMap<TypeId, UnsafePool>,
}

impl Context {
	#[inline(always)]
	#[must_use]
	pub fn create() -> NonNull<Context> {
		NonNull::from(Box::leak(Box::new(Context { pools: HashMap::new() })))
	}

	#[inline(always)]
	pub unsafe fn destroy(rt: NonNull<Context>) {
		unsafe { drop(Box::from_raw(rt.as_ptr())) }
	}

	#[inline]
	#[must_use]
	pub fn pool<T: 'static>(&mut self) -> &mut UnsafePool {
		self.pools.entry(TypeId::of::<T>()).or_insert_with(UnsafePool::new::<T>)
	}
}
