use winit::{event_loop::ActiveEventLoop, window::WindowId};

use crate::{component::Builder, standalone::state::WindowState};

pub struct Initializer<'a> {
	pub(crate) event_loop: &'a ActiveEventLoop,
	pub(crate) states: &'a mut Vec<WindowState>,
}

impl<'a> Initializer<'a> {
	pub fn create_window<B: Builder>(
		&mut self,
		window: crate::window::Window<B>,
	) -> Option<WindowId> {
		let (attributes, builder) = window.into_internal_data();
		let window = self.event_loop.create_window(attributes).ok()?;
		let id = window.id();

		// TODO: ここにコンポーネントを実体化する処理を入れる
		let state = WindowState { window, widget: builder.build() };
		self.states.push(state);
		Some(id)
	}
}
