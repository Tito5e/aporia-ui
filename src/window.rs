use winit::window::WindowAttributes;

use crate::component::Builder;

pub struct Window<B: Builder> {
	builder: B,

	title: String,
}

impl<B: Builder> Window<B> {
	pub fn mount(widget: B) -> Self {
		Window { builder: widget, title: Default::default() }
	}

	#[inline(always)]
	pub fn with_title(mut self, title: impl Into<String>) -> Self {
		self.title = title.into();
		self
	}

	#[inline(always)]
	pub(crate) fn into_internal_data(self) -> (WindowAttributes, B) {
		let attributes = WindowAttributes::default().with_title(self.title);

		(attributes, self.builder)
	}
}
