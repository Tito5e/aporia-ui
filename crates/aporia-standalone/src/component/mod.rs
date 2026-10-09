use aporia_core::geometry::{Constraint, Size};

use crate::{
	reactivity::{Signal, Source, effect::EffectHandle, scope::Scope},
	reconcile::{Reconcile, ReconcileCx, ReconcileEffect},
	widget::{Mount, Widget, WidgetHandle},
};

pub struct Component<R> {
	render: R,
}

impl<R: Render + 'static> Component<R> {
	pub fn new(render: R) -> Self {
		Self { render }
	}
}

pub struct View(WidgetHandle);

pub trait Render {
	type State;

	fn view(&self, cx: &mut ViewCx<Self::State>) -> View;
}

impl<S: 'static, R: Render<State = S> + 'static> Mount<S> for Component<R> {
	fn mount(self, cx: &mut ReconcileCx<S>) -> WidgetHandle {
		// Reserve heap storage for the component.
		// Because the component's Reconciler must point back to the component itself, the pointer address must be determined in advance.
		//
		// Therefore, use WidgetHandle::reserve instead of the regular WidgetHandle::new
		let reservation = WidgetHandle::reserve::<ComponentState<S, R>>();

		// The component needs an independent Scope to own the Signal defined in its view function and serve as the parent scope for its own Reconciler effect.
		let mut scope = cx.create_scope();

		// The component has a Reconciler effect responsible for triggering its own re-evaluation.
		let mut reconciler = cx.create_reconciler(&mut scope, reservation.as_ptr());
		let mut view_cx = ViewCx { cx, scope: &mut scope, reconciler: &mut reconciler };
		let child = self.render.view(&mut view_cx);

		reservation.write(ComponentState {
			component: self.render,
			scope,
			child: child.0,
			reconciler,
		})
	}
}

pub struct ViewCx<'a, 'b, S> {
	cx: &'a mut ReconcileCx<'b, S>,
	scope: &'a mut Scope,
	reconciler: &'a mut EffectHandle<ReconcileEffect<S>>,
}

impl<'a, 'b, S: 'static> ViewCx<'a, 'b, S> {
	#[inline(always)]
	#[must_use]
	pub fn render(&mut self, target: impl Mount<S>) -> View {
		View(target.mount(self.cx))
	}

	#[inline(always)]
	#[must_use]
	pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		self.scope.signal(value)
	}

	#[inline(always)]
	#[must_use]
	pub fn read<I: Source>(&mut self, source: I) -> I::Out {
		source.subscribe(self.reconciler)
	}

	#[inline(always)]
	#[must_use]
	pub fn global_state(&self) -> &S {
		self.cx.global_state()
	}
}

pub(crate) struct ComponentState<S, C: Render<State = S>> {
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
	pub(crate) reconciler: EffectHandle<ReconcileEffect<S>>,
	pub(crate) scope: Scope,
	pub(crate) component: C,
}

impl<S, C: Render<State = S>> Widget for ComponentState<S, C> {
	#[inline(always)]
	fn layout(&mut self, constraint: Constraint) -> Size {
		self.child.layout(constraint)
	}
}

impl<S, C: Render<State = S>> Reconcile for ComponentState<S, C> {
	type State = S;

	#[inline(always)]
	fn reconcile(&mut self, cx: &mut ReconcileCx<S>) {
		// Clear dependencies registered during the previous evaluation of the child widget.
		self.reconciler.unsubscribe_all();

		let mut view_cx = ViewCx { cx, scope: &mut self.scope, reconciler: &mut self.reconciler };
		let child = self.component.view(&mut view_cx);

		self.child = child.0;
	}
}
