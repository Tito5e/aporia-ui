use crate::{
	core::geometry::{Constraint, Size},
	reactivity::{
		context::Context,
		effect::{EffectHandle, ReconcilePhase},
	},
	storage::WidgetHandle,
	widget::Widget,
};

pub trait Builder {
	#[doc(hidden)]
	fn build(self) -> WidgetHandle;
}

pub struct NoChild;

impl Builder for NoChild {
	fn build(self) -> WidgetHandle {
		unreachable!("Called NoChild::build")
	}
}

pub trait Component: Sized {
	fn view(&self) -> WidgetHandle;
}

impl<C: Component + 'static> Builder for C {
	fn build(self) -> WidgetHandle {
		let reservation = WidgetHandle::reserve::<ComponentState<C>>();
		let scope =
			Context::create_reconcile_effect::<ComponentState<C>>(unsafe { reservation.as_ptr() });

		let before = Context::get_current_effect();
		Context::set_current_effect(Some(scope.as_ptr()));
		let child = self.view();
		Context::set_current_effect(before);

		let handle = reservation.write(ComponentState { component: self, scope, child });

		handle
	}
}

pub(crate) struct ComponentState<C: Component> {
	component: C,
	scope: EffectHandle,
	child: WidgetHandle,
}

impl<C: Component> Widget for ComponentState<C> {
	fn layout(&mut self, constraint: Constraint) -> Size {
		self.child.layout(constraint)
	}
}

impl<C: Component + 'static> ReconcilePhase for ComponentState<C> {
	fn on_reconcile_phase(&mut self) {
		let before = Context::get_current_effect();
		Context::set_current_effect(Some(self.scope.as_ptr()));
		let new_child = self.component.view();
		Context::set_current_effect(before);
		self.child = new_child;
	}
}
