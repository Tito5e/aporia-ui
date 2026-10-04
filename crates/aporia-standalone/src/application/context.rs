use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{
	reactivity::{Signal, effect::EffectHandle, runtime::Runtime, scope::Scope},
	reconcile::{Reconcile, ReconcileEffect, ReconcileQueue},
};

pub struct StandaloneCx {
	global_scope: ManuallyDrop<Scope>,
	rt: NonNull<Runtime>,

	/// dirty subscribers
	pub(crate) reconcile_queue: Box<ReconcileQueue>,
}

impl StandaloneCx {
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

	pub fn reconcile<T: Reconcile + 'static>(
		&mut self,
		scope: &mut Scope,
		ptr: NonNull<T>,
	) -> EffectHandle<ReconcileEffect> {
		let sink = NonNull::from(&mut self.reconcile_queue.header());
		scope.effect_with_sink(
			sink,
			ReconcileEffect::new(unsafe {
				NonNull::new_unchecked(ptr.as_ptr() as *mut dyn Reconcile)
			}),
		)
	}
}

impl Drop for StandaloneCx {
	fn drop(&mut self) {
		unsafe {
			ManuallyDrop::drop(&mut self.global_scope);
			Runtime::destroy(self.rt);
		}
	}
}
