use std::{any::TypeId, collections::HashMap, ptr::NonNull};

use crate::storage::UnsafePool;

pub struct Runtime {
	pub(crate) pools: HashMap<TypeId, UnsafePool>,
}

impl Runtime {
	pub fn create() -> NonNull<Runtime> {
		NonNull::from(Box::leak(Box::new(Runtime { pools: HashMap::new() })))
	}

	pub unsafe fn destroy(rt: NonNull<Runtime>) {
		// TODO: ScopeのDrop漏れ確認
		// ex: debug_assert!(pools.len == 0)
		unsafe { drop(Box::from_raw(rt.as_ptr())) }
	}
}
