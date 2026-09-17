use winit::window::{Window, WindowId};

pub(crate) struct WindowState {
	pub(crate) window: Window,
	//widget: Box<dyn Widget>,
}

impl WindowState {
	pub(crate) fn window_id(&self) -> WindowId {
		self.window.id()
	}

	pub(crate) fn request_redraw(&self) {
		self.window.request_redraw();
	}
}
