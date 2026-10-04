mod context;
mod initializer;
mod runner;
mod window;

pub use context::StandaloneCx;
pub use initializer::AppInitializer;
pub use initializer::StateInitializer;
use pollster::FutureExt;
pub(crate) use runner::AppRunner;

use std::error::Error;

use wgpu::DeviceDescriptor;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::RequestAdapterOptions;
use winit::event_loop::EventLoop;

pub struct NoGlobalState;

pub struct StandaloneApplication<S> {
	context: StandaloneCx,
	global_state: S,
}

impl Default for StandaloneApplication<NoGlobalState> {
	fn default() -> Self {
		let context = StandaloneCx::new();

		Self { context, global_state: NoGlobalState }
	}
}

impl<S> StandaloneApplication<S> {
	pub fn with_state<F: FnMut(&mut StateInitializer) -> S>(mut initializer: F) -> Self {
		let mut context = StandaloneCx::new();
		let mut state_cx = StateInitializer { context: &mut context };
		let global_state = initializer(&mut state_cx);

		Self { context, global_state }
	}

	pub fn run<F>(self, initializer: F) -> Result<(), Box<dyn Error>>
	where
		F: FnMut(&mut AppInitializer<S>),
	{
		let event_loop = EventLoop::new()?;

		let instance = Instance::new(InstanceDescriptor::new_without_display_handle());
		let adapter_options = RequestAdapterOptions::default();
		let adapter = instance.request_adapter(&adapter_options).block_on().unwrap();
		let device_desc = DeviceDescriptor::default();
		let (device, queue) = adapter.request_device(&device_desc).block_on().unwrap();

		let mut app: AppRunner<F, S> = AppRunner {
			initializer,
			states: Vec::new(),
			initialized: false,

			gpu_instance: instance,
			gpu_device: device,
			gpu_queue: queue,
			gpu_adapter: adapter,
			global_state: self.global_state,
			context: self.context,
		};

		event_loop.run_app(&mut app)?;
		Ok(())
	}
}
