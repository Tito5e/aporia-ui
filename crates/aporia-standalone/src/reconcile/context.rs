use std::ptr::NonNull;

use crate::{
	application::Registry,
	reactivity::{effect::EffectHandle, scope::Scope},
	reconcile::{Reconcile, ReconcileEffect},
};

pub struct ReconcileCx<'a, S> {
	cx: &'a mut Registry<S>,
	global_state: &'a S,
}

impl<'a, S: 'static> ReconcileCx<'a, S> {
	pub fn new(cx: &'a mut Registry<S>, global_state: &'a S) -> Self {
		Self { cx, global_state }
	}

	pub fn global_state(&self) -> &S {
		self.global_state
	}

	pub fn create_scope(&self) -> Scope {
		self.cx.create_scope()
	}

	#[inline]
	pub fn create_reconciler<T: Reconcile<State = S> + 'static>(
		&mut self,
		scope: &mut Scope,
		ptr: NonNull<T>,
	) -> EffectHandle<ReconcileEffect<S>> {
		self.cx.create_reconciler(scope, ptr)
	}
}
