use std::collections::HashMap;

use winit::{
	application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
	window::WindowId,
};

use crate::{
	application::{Registry, window::WindowState},
	reactivity::Signal,
	reconcile::ReconcileCx,
	renderer::GpuContext,
	widget::Mount,
};

pub struct App<F, S> {
	pub(crate) initialized: bool,
	pub(crate) initializer: F,

	pub(crate) windows: HashMap<WindowId, WindowState>,
	pub(crate) gpu: GpuContext,

	// Place GlobalState after WindowState and Renderer.
	pub(crate) global_state: S,

	// Place Registry after GlobalState.
	pub(crate) context: Registry<S>,
}

impl<F, S> App<F, S> {
	pub fn new(initializer: F, global_state: S, context: Registry<S>) -> Self {
		Self {
			// Application State Section
			initializer,
			global_state,
			context,

			// Application Engine Section
			initialized: false,
			windows: HashMap::new(),
			gpu: GpuContext::new(),
		}
	}
}

impl<F, S> ApplicationHandler for App<F, S>
where
	F: FnMut(&mut LaunchCx<S>),
{
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		if !self.initialized {
			let mut cx = LaunchCx {
				event_loop,
				windows: &mut self.windows,
				registry: &mut self.context,
				global_state: &mut self.global_state,
				gpu: &mut self.gpu,
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
				self.windows.remove(&window_id);

				if self.windows.is_empty() {
					event_loop.exit();
				}
			}
			WindowEvent::RedrawRequested => {
				let state = self.windows.get_mut(&window_id);

				if let Some(state) = state {
					state.draw(&self.gpu);
				}
			}
			WindowEvent::Resized(size) => {
				let state = self.windows.get_mut(&window_id);

				if let Some(state) = state {
					state.resize(&self.gpu, size);
				}
			}
			_ => {}
		}
	}

	fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
		// TODO: WindowStateごとに描画の途中の要素が存在しないかを確認し、存在する場合はrequest_redrawをコールする
	}

	fn suspended(&mut self, _: &winit::event_loop::ActiveEventLoop) {
		self.windows.clear();
		self.initialized = false;
	}
}

pub struct LaunchCx<'a, S> {
	global_state: &'a S,
	gpu: &'a GpuContext,
	event_loop: &'a ActiveEventLoop,
	windows: &'a mut HashMap<WindowId, WindowState>,
	registry: &'a mut Registry<S>,
}

impl<'a, S: 'static> LaunchCx<'a, S> {
	pub fn create_window<B: Mount<S>>(
		&mut self,
		window: crate::window::Window<S, B>,
	) -> Option<WindowId> {
		let (attributes, target) = window.into_internal_data();
		if let Ok(window) = self.event_loop.create_window(attributes) {
			let id = window.id();
			let mut cx = ReconcileCx::new(self.registry, self.global_state);
			let widget = target.mount(&mut cx);
			let state = WindowState::create(window, widget, self.gpu);
			self.windows.insert(id, state);

			return Some(id);
		}

		None
	}

	pub fn global_signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		self.registry.global_signal(value)
	}

	pub fn global_state(&self) -> &S {
		self.global_state
	}
}
