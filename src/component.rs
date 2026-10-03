use std::{mem::transmute, ptr::addr_of_mut};

use crate::{
	core::geometry::{Constraint, Size},
	reactivity::{Signal, Source, effect::EffectHandle, scope::Scope},
	reconcile::ReconcileEffect,
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

pub struct ViewCx<'a>(&'a mut Context, &'a mut Scope, &'a mut EffectHandle<ReconcileEffect>);

impl<'a> ViewCx<'a> {
	pub fn render(&mut self, builder: impl Builder) -> View {
		View(builder.build(self.0))
	}

	pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		self.1.signal(value)
	}

	pub fn read<T: Clone + 'static>(&mut self, signal: Signal<T>) -> T {
		let header = unsafe { &mut *addr_of_mut!((*self.2.ptr.as_ptr()).header) };
		signal.subscribe(header)
	}
}

pub trait Component {
	fn view(&self, cx: &mut ViewCx) -> View;
}

impl<C: Component + 'static> Builder for C {
	fn build(self, cx: &mut Context) -> WidgetHandle {
		let reservation = WidgetHandle::reserve::<ComponentState<C>>();
		let mut scope = cx.create_scope();
		let mut reconciler = cx.reconcile::<ComponentState<C>>(&mut scope, reservation.as_ptr());
		let mut view_cx = ViewCx(cx, &mut scope, &mut reconciler);
		let child = self.view(&mut view_cx);
		reservation.write(ComponentState { component: self, scope, child: child.0 })
	}
}

/// コンポーネントの定義順が非常に重要なのでいじらないように！
pub(crate) struct ComponentState<C: Component> {
	component: C,
	child: WidgetHandle,
	scope: Scope,
}

impl<C: Component> Widget for ComponentState<C> {
	fn layout(&mut self, constraint: Constraint) -> Size {
		self.child.layout(constraint)
	}
}
