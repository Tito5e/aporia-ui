use crate::{
	core::geometry::{Constraint, Size},
	reactivity::{Signal, Source, effect::EffectHandle, scope::Scope},
	reconcile::{Reconcile, ReconcileEffect},
	standalone::context::Context,
	storage::WidgetHandle,
	widget::Widget,
};

pub trait Builder {
	#[doc(hidden)]
	fn build(self, cx: &mut Context) -> WidgetHandle;
}

pub struct View(WidgetHandle);

pub struct ViewCx<'a> {
	ctx: &'a mut Context,
	scope: &'a mut Scope,
	reconciler: &'a mut EffectHandle<ReconcileEffect>,
}

impl<'a> ViewCx<'a> {
	#[inline(always)]
	#[must_use]
	pub fn render(&mut self, builder: impl Builder) -> View {
		View(builder.build(self.ctx))
	}

	#[inline(always)]
	#[must_use]
	pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		self.scope.signal(value)
	}

	#[inline(always)]
	#[must_use]
	pub fn read<S: Source>(&mut self, source: S) -> S::Out {
		source.subscribe(self.reconciler)
	}
}

pub trait Component {
	fn view(&self, cx: &mut ViewCx) -> View;
}

impl<C: Component + 'static> Builder for C {
	fn build(self, cx: &mut Context) -> WidgetHandle {
		// Reserve heap storage for the component.
		// Because the component's Reconciler must point back to the component itself, the pointer address must be determined in advance.
		//
		// Therefore, use WidgetHandle::reserve instead of the regular WidgetHandle::new
		let reservation = WidgetHandle::reserve::<ComponentState<C>>();

		// The component needs an independent Scope to own the Signal defined in its view function and serve as the parent scope for its own Reconciler effect.
		let mut scope = cx.create_scope();

		// The component has a Reconciler effect responsible for triggering its own re-evaluation.
		let mut reconciler = cx.reconcile::<ComponentState<C>>(&mut scope, reservation.as_ptr());
		let mut view_cx = ViewCx { ctx: cx, scope: &mut scope, reconciler: &mut reconciler };
		let child = self.view(&mut view_cx);

		reservation.write(ComponentState { component: self, scope, child: child.0, reconciler })
	}
}

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
	child: WidgetHandle,
	reconciler: EffectHandle<ReconcileEffect>,
	scope: Scope,
	component: C,
}

impl<C: Component> Widget for ComponentState<C> {
	#[inline(always)]
	fn layout(&mut self, constraint: Constraint) -> Size {
		self.child.layout(constraint)
	}
}

impl<C: Component> Reconcile for ComponentState<C> {
	#[inline(always)]
	fn reconcile(&mut self, cx: &mut Context) {
		// Clear dependencies registered during the previous evaluation of the child widget.
		self.reconciler.unsubscribe_all();

		let mut view_cx =
			ViewCx { ctx: cx, scope: &mut self.scope, reconciler: &mut self.reconciler };
		let child = self.component.view(&mut view_cx);

		self.child = child.0;
	}
}
