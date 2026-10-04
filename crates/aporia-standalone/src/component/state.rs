use aporia_core::geometry::{Constraint, Size};

use crate::{
	component::{Component, ViewCx},
	context::StandaloneCx,
	reactivity::{effect::EffectHandle, scope::Scope},
	reconcile::{Reconcile, ReconcileCx, ReconcileEffect},
	widget::{Widget, WidgetHandle},
};

pub(crate) struct ComponentState<C: Component> {
	// ComponentState must maintain a specific field drop order.
	//
	// WidgetHandle and Scope have custom Drop implementations and impose invariants that must be upheld by the caller.
	// These invariants are fairly complex, but they can be safely satisfied by ensuring that fields are dropped in the reverse order of their declaration.
	//
	// Therefore, the required drop order is:
	//
	// child -> reconciler -> scope -> component
	//
	// Do not change the field declaration order.
	pub(crate) child: WidgetHandle,
	pub(crate) reconciler: EffectHandle<ReconcileEffect>,
	pub(crate) scope: Scope,
	pub(crate) component: C,
}

impl<C: Component> Widget for ComponentState<C> {
	#[inline(always)]
	fn layout(&mut self, constraint: Constraint) -> Size {
		self.child.layout(constraint)
	}
}

impl<C: Component> Reconcile for ComponentState<C> {
	#[inline(always)]
	fn reconcile(&mut self, cx: &mut ReconcileCx) {
		// Clear dependencies registered during the previous evaluation of the child widget.
		self.reconciler.unsubscribe_all();

		let mut view_cx =
			ViewCx { ctx: cx.cx, scope: &mut self.scope, reconciler: &mut self.reconciler };
		let child = self.component.view(&mut view_cx);

		self.child = child.0;
	}
}
