mod state;

pub(crate) use state::ComponentState;

use crate::{
	reactivity::{Signal, Source, effect::EffectHandle, scope::Scope},
	reconcile::{ReconcileCx, ReconcileEffect},
	widget::{Mount, WidgetHandle},
};

pub struct View(WidgetHandle);

pub trait Component<S> {
	fn view(&self, cx: &mut ViewCx<S>) -> View;
}

impl<S: 'static, C: Component<S> + 'static> Mount<S> for C {
	fn mount(self, cx: &mut ReconcileCx<S>) -> WidgetHandle {
		// Reserve heap storage for the component.
		// Because the component's Reconciler must point back to the component itself, the pointer address must be determined in advance.
		//
		// Therefore, use WidgetHandle::reserve instead of the regular WidgetHandle::new
		let reservation = WidgetHandle::reserve::<ComponentState<S, C>>();

		// The component needs an independent Scope to own the Signal defined in its view function and serve as the parent scope for its own Reconciler effect.
		let mut scope = cx.create_scope();

		// The component has a Reconciler effect responsible for triggering its own re-evaluation.
		let mut reconciler = cx.create_reconciler(&mut scope, reservation.as_ptr());
		let mut view_cx = ViewCx { cx, scope: &mut scope, reconciler: &mut reconciler };
		let child = self.view(&mut view_cx);

		reservation.write(ComponentState { component: self, scope, child: child.0, reconciler })
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
