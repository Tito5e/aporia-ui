mod runner;

use std::error::Error;

use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, EventLoop},
	window::{Window, WindowId},
};

use crate::component::Builder;

pub struct Initializer<'a> {
	event_loop: &'a ActiveEventLoop,
	windows: &'a mut Vec<Window>,
}

impl<'a> Initializer<'a> {
	pub fn create_window<B: Builder>(
		&mut self,
		window: crate::window::Window<B>,
	) -> Option<WindowId> {
		let (attributes, builder) = window.into_internal_data();
		let window = self.event_loop.create_window(attributes).ok()?;
		let id = window.id();
		self.windows.push(window);
		Some(id)
	}
}

pub struct StandaloneApplication;

impl StandaloneApplication {
	pub fn new() -> Self {
		Self
	}

	pub fn run<F>(self, initializer: F) -> Result<(), Box<dyn Error>>
	where
		F: FnMut(&mut Initializer),
	{
		let event_loop = EventLoop::new()?;
		let mut app = AppRunner { initializer, windows: Vec::new() };

		event_loop.run_app(&mut app)?;
		Ok(())
	}
}

struct AppRunner<F> {
	initializer: F,
	windows: Vec<Window>,
}

impl<F> ApplicationHandler for AppRunner<F>
where
	F: FnMut(&mut Initializer),
{
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		if self.windows.is_empty() {
			let mut cx = Initializer { event_loop, windows: &mut self.windows };
			(self.initializer)(&mut cx);
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
				self.windows.retain(|w| w.id() != window_id);

				if self.windows.is_empty() {
					event_loop.exit();
				}
			}
			WindowEvent::RedrawRequested => {
				if let Some(window) = self.windows.iter().find(|w| w.id() == window_id) {
					window.request_redraw();
				}
			}
			_ => {}
		}
	}
}
