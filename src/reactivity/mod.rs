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

pub use mapped::Mapped;
pub use signal::Signal;

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

#[cfg(test)]
mod tests {
	use crate::reactivity::{Mapped, Read, runtime::Runtime, scope::Scope};

	#[test]
	fn get_returns_reference_without_clone() {
		struct NoClone(String);
		let rt = Runtime::create();
		{
			let mut scope = Scope::new(rt);
			let signal = scope.signal(NoClone("hello".to_string()));
			assert_eq!(signal.read().0, "hello");
			let len = Mapped::new((signal,), |(v,)| v.0.len());
			assert_eq!(len.get(), 5);
		}
		unsafe {
			Runtime::destroy(rt);
		}
	}
}
