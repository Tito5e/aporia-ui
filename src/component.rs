use crate::{
	core::geometry::{Constraint, Size},
	reactivity::{effect::EffectHandle, scope::Scope},
	standalone::context::Context,
	storage::WidgetHandle,
	widget::Widget,
};

pub trait Builder {
	#[doc(hidden)]
	fn build(self, cx: &mut Context) -> WidgetHandle;
}

pub struct NoChild;

impl Builder for NoChild {
	fn build(self, cx: &mut Context) -> WidgetHandle {
		unreachable!("Called NoChild::build")
	}
}

pub struct View(WidgetHandle);

pub struct ViewCx<'a>(&'a mut Context);

pub trait Component {
	fn view(&self, cx: &mut ViewCx) -> View;
}

impl<C: Component + 'static> Builder for C {
	fn build(self, cx: &mut Context) -> WidgetHandle {
		let reservation = WidgetHandle::reserve::<ComponentState<C>>();
		let scope = cx.create_scope();
		let mut view_cx = ViewCx(cx);
		let child = self.view(&mut view_cx);
		drop(view_cx);
		reservation.write(ComponentState { component: self, scope, child: child.0 })
	}
}

pub(crate) struct ComponentState<C: Component> {
	component: C,
	scope: Scope,
	child: WidgetHandle,
}

impl<C: Component> Widget for ComponentState<C> {
	fn layout(&mut self, constraint: Constraint) -> Size {
		self.child.layout(constraint)
	}
}
