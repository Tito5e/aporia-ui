use std::{ffi::c_void, mem::ManuallyDrop, ptr::NonNull};

use crate::{
	reactivity::{Signal, effect::EffectHandle, runtime::Runtime, scope::Scope},
	reconcile::{ReconcileEffect, ReconcileQueue},
};

// TODO: パフォーマンス計測が必要
pub struct Context {
	global_scope: ManuallyDrop<Scope>,
	rt: NonNull<Runtime>,

	/// dirty subscribers
	pub(crate) reconcile_queue: Box<ReconcileQueue>,
}

impl Context {
	pub(crate) fn new() -> Self {
		let rt = Runtime::create();
		let global_scope = ManuallyDrop::new(Scope::new(rt));
		Self { rt, global_scope, reconcile_queue: Box::new(ReconcileQueue::new()) }
	}

	pub fn create_scope(&self) -> Scope {
		Scope::new(self.rt)
	}

	pub fn global_signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		self.global_scope.signal(value)
	}

	pub fn reconcile<T: 'static>(
		&mut self,
		scope: &mut Scope,
		ptr: NonNull<c_void>,
	) -> EffectHandle<ReconcileEffect> {
		let sink = NonNull::from(&mut self.reconcile_queue.header());
		scope.effect_with_sink(sink, ReconcileEffect {})
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			ManuallyDrop::drop(&mut self.global_scope);
			Runtime::destroy(self.rt);
		}
	}
}
