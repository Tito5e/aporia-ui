use wgpu::{
	Device, Instance, Queue, Surface, SurfaceConfiguration, SurfaceTargetUnsafe, TextureFormat,
};
use winit::{
	dpi::PhysicalSize,
	window::{Window, WindowId},
};

use crate::component::WidgetHandle;

pub(crate) struct WindowState {
	pub(crate) surface: Surface<'static>,
	pub(crate) surface_format: TextureFormat,
	pub(crate) surface_config: SurfaceConfiguration,
	pub(crate) gpu_device: Device,
	pub(crate) gpu_instance: Instance,
	pub(crate) gpu_queue: Queue,
	pub(crate) size: PhysicalSize<u32>,

	pub(crate) window: Window,
	pub(crate) widget: WidgetHandle,
}

impl WindowState {
	pub(crate) fn window_id(&self) -> WindowId {
		self.window.id()
	}

	pub(crate) fn request_redraw(&self) {
		self.window.request_redraw();
	}

	pub(crate) fn configure_surface(&self) {
		let surface_config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: self.surface_format,
			color_space: wgpu::SurfaceColorSpace::Auto,
			// Request compatibility with the sRGB-format texture view we‘re going to create later.
			view_formats: vec![self.surface_format.add_srgb_suffix()],
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			width: self.size.width,
			height: self.size.height,
			desired_maximum_frame_latency: 2,
			present_mode: wgpu::PresentMode::AutoVsync,
		};
		self.surface.configure(&self.gpu_device, &surface_config);
	}

	pub(crate) fn draw(&mut self) {
		let surface_texture = match self.surface.get_current_texture() {
			wgpu::CurrentSurfaceTexture::Success(texture) => texture,
			wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
			wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
				drop(texture);
				self.configure_surface();
				return;
			}
			wgpu::CurrentSurfaceTexture::Outdated => {
				self.configure_surface();
				return;
			}
			wgpu::CurrentSurfaceTexture::Validation => {
				unreachable!("No error scope registered, so validation errors will panic")
			}
			wgpu::CurrentSurfaceTexture::Lost => {
				self.surface = unsafe {
					self.gpu_instance
						.create_surface_unsafe(
							SurfaceTargetUnsafe::from_window(&self.window).unwrap(),
						)
						.unwrap()
				};
				self.configure_surface();
				return;
			}
		};
		let texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
			// Without add_srgb_suffix() the image we will be working with
			// might not be "gamma correct".
			format: Some(self.surface_format.add_srgb_suffix()),
			..Default::default()
		});

		// Renders a GREEN screen
		let mut encoder = self.gpu_device.create_command_encoder(&Default::default());
		// Create the renderpass which will clear the screen.
		let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &texture_view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color::RED),
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
			multiview_mask: None,
		});

		// If you wanted to call any drawing commands, they would go here.

		// End the renderpass.
		drop(renderpass);

		// Submit the command in the queue to execute
		self.gpu_queue.submit([encoder.finish()]);
		self.window.pre_present_notify();
		self.gpu_queue.present(surface_texture);
	}
}
