use winit::window::{Window, WindowId};

use crate::component::WidgetHandle;

pub(crate) struct WindowState {
	pub(crate) window: Window,
	pub(crate) widget: WidgetHandle,
}

impl WindowState {
	pub(crate) fn window_id(&self) -> WindowId {
		self.window.id()
	}

	pub(crate) fn request_redraw(&self) {
		self.window.request_redraw();
	}
}
