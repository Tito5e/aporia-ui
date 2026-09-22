use log::debug;
use wgpu::{
	Adapter, Device, Instance, Queue, SurfaceColorSpace, SurfaceConfiguration, SurfaceTargetUnsafe,
};
use winit::{event_loop::ActiveEventLoop, window::WindowId};

use crate::{component::Builder, standalone::state::WindowState};

pub struct Initializer<'a> {
	pub(crate) gpu_instance: &'a mut Instance,
	pub(crate) gpu_adapter: &'a mut Adapter,
	pub(crate) gpu_device: &'a mut Device,
	pub(crate) gpu_queue: &'a mut Queue,

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
		debug!("Building Widget");
		let widget_handle = builder.build();
		debug!("Complete");

		let surface = unsafe {
			self.gpu_instance.create_surface_unsafe(
				SurfaceTargetUnsafe::from_display_and_window(&window, &window).unwrap(),
			)
		}
		.unwrap();
		let cap = surface.get_capabilities(&self.gpu_adapter);
		let surface_format = cap.formats[0];
		let size = window.inner_size();
		let surface_config = SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
			color_space: SurfaceColorSpace::Auto,
			desired_maximum_frame_latency: 2,
			alpha_mode: Default::default(),
			view_formats: vec![surface_format.add_srgb_suffix()],
		};
		surface.configure(self.gpu_device, &surface_config);

		let state = WindowState {
			window,
			widget: widget_handle,
			size,
			surface,
			surface_format,
			surface_config,
			gpu_device: self.gpu_device.clone(),
			gpu_instance: self.gpu_instance.clone(),
			gpu_queue: self.gpu_queue.clone(),
		};
		self.states.push(state);
		Some(id)
	}
}
