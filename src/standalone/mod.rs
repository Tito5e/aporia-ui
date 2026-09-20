use std::error::Error;

use pollster::FutureExt;
use wgpu::{DeviceDescriptor, Instance, InstanceDescriptor, RequestAdapterOptions};
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
		let mut app = StandaloneApplication::initialize(initializer);

		event_loop.run_app(&mut app)?;
		Ok(())
	}

	fn initialize<F: FnMut(&mut Initializer)>(initializer: F) -> AppRunner<F> {
		let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
		let adapter_options = RequestAdapterOptions::default();
		let adapter = instance.request_adapter(&adapter_options).block_on().unwrap();
		let device_desc = DeviceDescriptor::default();
		let (device, queue) = adapter.request_device(&device_desc).block_on().unwrap();

		AppRunner {
			initializer,
			states: Vec::new(),
			initialized: false,
			gpu_instance: instance,
			gpu_device: device,
			gpu_queue: queue,
			gpu_adapter: adapter,
		}
	}
}
