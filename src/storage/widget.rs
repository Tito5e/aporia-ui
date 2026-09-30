use crate::{
	core::geometry::{Constraint, Size},
	reactivity::context::Context,
	storage::Key,
	widget::Widget,
};

pub struct WidgetHandle {
	index: Key,
	vtable: &'static WidgetVTable,
}

struct WidgetVTable {
	layout: unsafe fn(Key, Constraint) -> Size,
	drop_fn: unsafe fn(Key),
}

impl WidgetVTable {
	const fn build<T: Widget + 'static>() -> &'static Self {
		&Self {
			layout: |mut index, constraint| {
				let widget = unsafe { Context::get_widget_mut::<T>(&mut index) };
				widget.layout(constraint)
			},
			drop_fn: |index| {
				let widget = unsafe { Context::remove_widget::<T>(index) };
				drop(widget);
			},
		}
	}
}

impl WidgetHandle {
	pub(crate) fn new<T: Widget + 'static>(index: Key) -> Self {
		Self { index, vtable: WidgetVTable::build::<T>() }
	}

	pub fn layout(&self, constraint: Constraint) -> Size {
		unsafe { (self.vtable.layout)(self.index, constraint) }
	}
}

impl Drop for WidgetHandle {
	fn drop(&mut self) {
		unsafe { (self.vtable.drop_fn)(self.index) }
	}
}
