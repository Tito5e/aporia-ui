use std::any::TypeId;

use crate::{
	reactivity::{effect::EffectHeader, runtime::Runtime},
	storage::UnsafePool,
};

pub(crate) mod effect;
pub(crate) mod mapped;
pub(crate) mod runtime;
pub(crate) mod scope;
pub(crate) mod signal;
pub(crate) mod sink;

pub const NONE: u32 = u32::MAX;

pub trait Subscribe {
	fn subscribe(&self, effect: &mut EffectHeader);
}

pub trait Read {
	type Out<'a>
	where
		Self: 'a;

	fn read(&self) -> Self::Out<'_>;
}

pub(crate) unsafe fn pool<T: 'static>(rt: *mut Runtime) -> *mut UnsafePool {
	unsafe {
		(*rt).pools.entry(TypeId::of::<T>()).or_insert_with(UnsafePool::new::<T>) as *mut UnsafePool
	}
}
