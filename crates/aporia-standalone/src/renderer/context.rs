use wgpu::{
	DeviceDescriptor, Instance, PowerPreference, RequestAdapterOptions, RequestDeviceError,
};

pub struct GpuContext {
	pub instance: wgpu::Instance,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
}

impl GpuContext {
	pub async fn new_async() -> Result<Self, RequestDeviceError> {
		let instance = Instance::default();
		let adapter = instance
			.request_adapter(&RequestAdapterOptions {
				power_preference: PowerPreference::HighPerformance,
				compatible_surface: None,
				force_fallback_adapter: false,
				apply_limit_buckets: false,
			})
			.await
			.expect("Failed to find an appropriate GPU adapter.");

		let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;

		Ok(Self { instance, adapter, device, queue })
	}

	pub fn new() -> Self {
		pollster::block_on(Self::new_async()).expect("Failed to create GpuContext.")
	}
}
