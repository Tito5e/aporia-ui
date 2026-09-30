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

pub trait Render {
	type Output: Builder;
	#[doc(hidden)]
	fn render(&mut self) -> Self::Output;
}

impl<F, B> Render for F
where
	F: FnMut() -> B,
	B: Builder,
{
	type Output = B;
	fn render(&mut self) -> B {
		self()
	}
}

pub trait Component {
	fn view(self) -> impl Render + 'static;
}

impl<C: Component + 'static> Builder for C {
	fn build(self) -> WidgetHandle {
		build_component(self.view())
	}
}

fn build_component<R: Render + 'static>(mut render: R) -> WidgetHandle {
	let reservation = Context::reserve_widget::<ComponentState<R>>();
	let scope =
		Context::create_reconcile_effect::<ComponentState<R>>(unsafe { reservation.as_key() });

	let before = Context::get_current_effect();
	Context::set_current_effect(Some(scope.as_ptr()));
	let child_builder = render.render();
	Context::set_current_effect(before);

	let child = child_builder.build();
	reservation.write(ComponentState { render, child, scope })
}

pub(crate) struct ComponentState<R: Render> {
	render: R,
	scope: EffectHandle,
	child: WidgetHandle,
}

impl<R: Render> Widget for ComponentState<R> {
	fn layout(&mut self, constraint: Constraint) -> Size {
		self.child.layout(constraint)
	}
}

impl<R: Render> ReconcilePhase for ComponentState<R> {
	fn on_reconcile_phase(&mut self) {
		let before_scope = Context::get_current_effect();
		Context::set_current_effect(Some(self.scope.as_ptr()));
		let child_builder = self.render.render();
		Context::set_current_effect(before_scope);
		self.child = child_builder.build();
	}
}
