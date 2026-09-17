use winit::{application::ApplicationHandler, event::WindowEvent};

use crate::standalone::{initializer::Initializer, state::WindowState};

pub(crate) struct AppRunner<F> {
	pub(crate) initializer: F,
	pub(crate) states: Vec<WindowState>,
	pub(crate) initialized: bool,
}

impl<F> ApplicationHandler for AppRunner<F>
where
	F: FnMut(&mut Initializer),
{
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		if !self.initialized {
			let mut cx = Initializer { event_loop, states: &mut self.states };
			(self.initializer)(&mut cx);

			self.initialized = true;
		}
	}

	fn window_event(
		&mut self,
		event_loop: &winit::event_loop::ActiveEventLoop,
		window_id: winit::window::WindowId,
		event: winit::event::WindowEvent,
	) {
		match event {
			WindowEvent::CloseRequested => {
				self.states.retain(|w| w.window_id() != window_id);

				if self.states.is_empty() {
					event_loop.exit();
				}
			}
			_ => {}
		}
	}

	fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
		// TODO: WindowStateごとに描画の途中の要素が存在しないかを確認し、存在する場合はrequest_redrawをコールする
	}

	fn suspended(&mut self, _: &winit::event_loop::ActiveEventLoop) {
		self.states.clear();
		self.initialized = false;
	}
}
