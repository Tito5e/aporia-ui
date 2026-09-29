use crate::{
	reactivity::{
		context::Context,
		effect::{BuildPhase, ReconcileHandle},
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

pub trait Component {
	fn view(&self) -> impl Builder;
}

impl<C: Component + 'static> Builder for C {
	fn build(self) -> WidgetHandle {
		let reservation = Context::reserve_widget::<ComponentState<C>>();
		let scope = Context::create_build_effect::<ComponentState<C>>();
		let before_scope = Context::get_current_effect();
		Context::set_current_effect(Some(scope));
		let child_builder = self.view();
		Context::set_current_effect(before_scope);
		let child = child_builder.build();

		let state = ComponentState { component: self, child, scope };
		let handle = reservation.write(state);

		handle
	}
}

pub(crate) struct ComponentState<C: Component> {
	component: C,
	child: WidgetHandle,
	scope: ReconcileHandle,
}

impl<C: Component> Widget for ComponentState<C> {
	fn layout(
		&mut self,
		constraint: crate::core::geometry::Constraint,
	) -> crate::core::geometry::Size {
		self.child.layout(constraint)
	}
}

impl<C: Component> BuildPhase for ComponentState<C> {
	fn on_build_phase(&mut self) {
		let before_scope = Context::get_current_effect();
		Context::set_current_effect(Some(self.scope));
		let child_builder = self.component.view();
		Context::set_current_effect(before_scope);
		let child = child_builder.build();
		self.child = child;
	}
}
