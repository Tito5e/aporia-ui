use std::any::TypeId;

use crate::reactivity::{effect::EffectHeader, runtime::Runtime};

use unsafe_pool::UnsafePool;

pub(crate) mod r#const;
pub(crate) mod effect;
pub(crate) mod mapped;
pub(crate) mod runtime;
pub(crate) mod scope;
pub(crate) mod signal;
pub(crate) mod sink;

pub use r#const::Const;
pub use mapped::Mapped;
pub use mapped::MappedCx;
pub use signal::Signal;

pub const NONE: u32 = u32::MAX;

pub trait Source {
	type Out;

	fn read(&self) -> Self::Out;
	fn subscribe(&self, e: &mut EffectHeader) -> Self::Out;
}

pub(crate) unsafe fn pool<T: 'static>(rt: *mut Runtime) -> *mut UnsafePool {
	unsafe {
		(*rt).pools.entry(TypeId::of::<T>()).or_insert_with(UnsafePool::new::<T>) as *mut UnsafePool
	}
}
