use std::ptr::addr_of_mut;

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

pub struct NoChild;

impl Builder for NoChild {
	fn build(self, cx: &mut Context) -> WidgetHandle {
		unreachable!("Called NoChild::build")
	}
}

pub struct View(WidgetHandle);

pub struct ViewCx<'a> {
	ctx: &'a mut Context,
	scope: &'a mut Scope,
	reconciler: &'a mut EffectHandle<ReconcileEffect>,
}

impl<'a> ViewCx<'a> {
	pub fn render(&mut self, builder: impl Builder) -> View {
		View(builder.build(self.ctx))
	}

	pub fn signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		self.scope.signal(value)
	}

	pub fn read<T: Clone + 'static>(&mut self, signal: Signal<T>) -> T {
		let header = unsafe { &mut *addr_of_mut!((*self.reconciler.ptr.as_ptr()).header) };
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
		let mut view_cx = ViewCx { ctx: cx, scope: &mut scope, reconciler: &mut reconciler };
		let child = self.view(&mut view_cx);
		reservation.write(ComponentState { component: self, scope, child: child.0, reconciler })
	}
}

pub(crate) struct ComponentState<C: Component> {
	// コンポーネントの定義順が非常に重要なのでいじらないように！
	component: C,
	child: WidgetHandle,
	scope: Scope,
	reconciler: EffectHandle<ReconcileEffect>,
}

impl<C: Component> Widget for ComponentState<C> {
	fn layout(&mut self, constraint: Constraint) -> Size {
		self.child.layout(constraint)
	}
}

impl<C: Component> Reconcile for ComponentState<C> {
	fn reconcile(&mut self, cx: &mut Context) {
		self.reconciler.unsubscribe_all();
		let mut view_cx =
			ViewCx { ctx: cx, scope: &mut self.scope, reconciler: &mut self.reconciler };
		let child = self.component.view(&mut view_cx);
		self.child = child.0;
	}
}
