use std::ptr::NonNull;

use crate::{
	reactivity::{runtime::Runtime, scope::Scope},
	reconcile::ReconcileQueue,
};

// TODO: パフォーマンス計測が必要
pub(crate) struct Context {
	rt: NonNull<Runtime>,

	/// dirty subscribers
	pub(crate) reconcile_queue: Box<ReconcileQueue>,
}

impl Context {
	pub(crate) fn new() -> Self {
		Self { rt: Runtime::create(), reconcile_queue: Box::new(ReconcileQueue::new()) }
	}

	pub fn create_scope(&self) -> Scope {
		Scope::new(self.rt)
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			Runtime::destroy(self.rt);
		}
	}
}
