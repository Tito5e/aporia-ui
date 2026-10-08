use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::{
	reactivity::{Signal, effect::EffectHandle, runtime::Runtime, scope::Scope},
	reconcile::{Reconcile, ReconcileEffect, ReconcileQueue},
};

pub struct Registry<S> {
	global_scope: ManuallyDrop<Scope>,
	rt: NonNull<Runtime>,

	/// dirty subscribers
	pub(crate) reconcile_queue: Box<ReconcileQueue<S>>,
}

impl<S: 'static> Registry<S> {
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

	pub fn create_reconciler<T: Reconcile<S> + 'static>(
		&mut self,
		scope: &mut Scope,
		ptr: NonNull<T>,
	) -> EffectHandle<ReconcileEffect<S>> {
		let sink = NonNull::from(&mut self.reconcile_queue.header());
		scope.effect_with_sink(
			sink,
			ReconcileEffect::new(unsafe {
				NonNull::new_unchecked(ptr.as_ptr() as *mut dyn Reconcile<S>)
			}),
		)
	}
}

impl<S> Drop for Registry<S> {
	fn drop(&mut self) {
		unsafe {
			ManuallyDrop::drop(&mut self.global_scope);
			Runtime::destroy(self.rt);
		}
	}
}
