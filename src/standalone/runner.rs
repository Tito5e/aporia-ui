use wgpu::{Adapter, Device, Instance, Queue};
use winit::{application::ApplicationHandler, event::WindowEvent};

use crate::standalone::{initializer::Initializer, state::WindowState};

pub(crate) struct AppRunner<F> {
	pub(crate) initializer: F,
	pub(crate) states: Vec<WindowState>,
	pub(crate) initialized: bool,
	pub(crate) gpu_instance: Instance,
	pub(crate) gpu_device: Device,
	pub(crate) gpu_queue: Queue,
	pub(crate) gpu_adapter: Adapter,
}

impl<F> ApplicationHandler for AppRunner<F>
where
	F: FnMut(&mut Initializer),
{
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		if !self.initialized {
			let mut cx = Initializer {
				event_loop,
				states: &mut self.states,
				gpu_instance: &mut self.gpu_instance,
				gpu_adapter: &mut self.gpu_adapter,
				gpu_device: &mut self.gpu_device,
				gpu_queue: &mut self.gpu_queue,
			};
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
			WindowEvent::RedrawRequested => {
				let state = self.states.iter_mut().find(|w| w.window_id() == window_id);

				if let Some(state) = state {
					state.draw();
				}
			}
			WindowEvent::Resized(size) => {
				let state = self.states.iter_mut().find(|w| w.window_id() == window_id);

				if let Some(state) = state {
					state.size = size;
					state.configure_surface();
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
