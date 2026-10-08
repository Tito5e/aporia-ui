mod app;
mod registry;
mod window;

pub(crate) use app::App;
pub use registry::Registry;

use std::error::Error;

use winit::event_loop::{ControlFlow, EventLoop};

use crate::{application::app::LaunchCx, reactivity::Signal};

pub struct StandaloneApplication<S> {
	registry: Registry<S>,
	global_state: S,
}

impl Default for StandaloneApplication<()> {
	fn default() -> Self {
		let registry = Registry::new();

		Self { registry, global_state: () }
	}
}

impl<S: 'static> StandaloneApplication<S> {
	pub fn with_state<F: FnMut(&mut InitializeCx<S>) -> S>(mut initializer: F) -> Self {
		let mut registry = Registry::new();
		let mut cx = InitializeCx { registry: &mut registry };
		let global_state = initializer(&mut cx);

		Self { registry, global_state }
	}

	pub fn run<F>(self, initializer: F) -> Result<(), Box<dyn Error>>
	where
		F: FnMut(&mut LaunchCx<S>),
	{
		let event_loop = EventLoop::new()?;
		event_loop.set_control_flow(ControlFlow::Wait);

		let mut app: App<F, S> = App::new(initializer, self.global_state, self.registry);

		event_loop.run_app(&mut app)?;
		Ok(())
	}
}

pub struct InitializeCx<'a, S> {
	registry: &'a mut Registry<S>,
}

impl<'a, S: 'static> InitializeCx<'a, S> {
	#[inline]
	#[must_use]
	pub fn global_signal<T: 'static>(&mut self, value: T) -> Signal<T> {
		self.registry.global_signal(value)
	}
}
