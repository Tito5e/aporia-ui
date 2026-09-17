use std::error::Error;

use winit::event_loop::EventLoop;

use crate::standalone::{initializer::Initializer, runner::AppRunner};

pub(crate) mod initializer;
pub(crate) mod runner;
pub(crate) mod state;

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
		let mut app = AppRunner { initializer, states: Vec::new(), initialized: false };

		event_loop.run_app(&mut app)?;
		Ok(())
	}
}
