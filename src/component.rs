use std::ptr;

use crate::{
	reactivity::{
		allocator::HeapAllocator,
		context::Context,
		effect::{BuildEffect, BuildPhase},
	},
	widget::Widget,
};

pub trait Builder {
	#[doc(hidden)]
	fn build(self) -> WidgetHandle;
}

pub struct WidgetHandle(pub(crate) *mut dyn Widget);

impl WidgetHandle {
	pub fn layout(
		&self,
		constraint: crate::core::geometry::Constraint,
	) -> crate::core::geometry::Size {
		let widget = unsafe { &mut *self.0 };
		widget.layout(constraint)
	}
}

impl Drop for WidgetHandle {
	fn drop(&mut self) {
		Context::free_widget(self);
	}
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
		// TODO: コンポーネントの配置をBoxを用いた危険な実装からヒープアロケーターを用いたものに変更する
		let raw_ptr = unsafe {
			HeapAllocator::alloc(size_of::<ComponentState<C>>(), align_of::<ComponentState<C>>())
		};
		let scope =
			BuildEffect::new_validated(raw_ptr as *mut ComponentState<C> as *mut dyn BuildPhase);
		let before_scope = Context::get_current_effect();
		Context::set_current_effect(Some(scope));
		let child_builder = self.view();
		Context::set_current_effect(before_scope);
		let child = child_builder.build();

		let state = ComponentState { component: self, child };
		unsafe { ptr::write(raw_ptr as *mut ComponentState<C>, state) };
		let handle = WidgetHandle(raw_ptr as *mut ComponentState<C>);

		handle
	}
}

pub(crate) struct ComponentState<C: Component> {
	component: C,
	child: WidgetHandle,
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
		todo!()
	}
}
