use wgpu::{
	CompositeAlphaMode, PresentMode, Surface, SurfaceColorSpace, SurfaceConfiguration,
	SurfaceTargetUnsafe, TextureFormat, TextureUsages,
};
use winit::{
	dpi::PhysicalSize,
	window::{Window, WindowId},
};

use crate::{
	component::Component,
	reconcile::ReconcileCx,
	renderer::{GpuContext, Renderer},
	widget::{Mount as _, WidgetHandle},
};

pub(crate) struct WindowState {
	pub(crate) renderer: Renderer,
	pub(crate) surface: Surface<'static>,
	pub(crate) surface_format: TextureFormat,
	pub(crate) size: PhysicalSize<u32>,

	pub(crate) window: Window,
	pub(crate) widget: WidgetHandle,
}

impl WindowState {
	pub(crate) fn create<S>(
		window: Window,
		widget: WidgetHandle,
		gpu_context: &GpuContext,
		global_state: &S,
	) -> Self {
		let size = window.inner_size();

		let surface_target = unsafe { SurfaceTargetUnsafe::from_window(&window).unwrap() };
		let surface =
			unsafe { gpu_context.instance.create_surface_unsafe(surface_target).unwrap() };

		let capabilities = surface.get_capabilities(&gpu_context.adapter);
		let format = capabilities.formats[0];

		let renderer = Renderer::new(&gpu_context.device, format);

		let state = WindowState { window, widget, size, surface, surface_format: format, renderer };

		state.configure_surface(gpu_context);
		state
	}

	fn configure_surface(&self, gpu_context: &GpuContext) {
		let config = SurfaceConfiguration {
			usage: TextureUsages::RENDER_ATTACHMENT,
			format: self.surface_format,
			color_space: SurfaceColorSpace::Auto,
			view_formats: vec![self.surface_format.add_srgb_suffix()],
			alpha_mode: CompositeAlphaMode::Auto,
			width: self.size.width,
			height: self.size.height,
			desired_maximum_frame_latency: 2,
			present_mode: PresentMode::AutoVsync,
		};
		self.surface.configure(&gpu_context.device, &config);
	}

	pub fn resize(&mut self, gpu_context: &GpuContext, size: PhysicalSize<u32>) {
		if size.width > 0 && size.height > 0 {
			self.size = size;
			self.configure_surface(gpu_context);
		}
	}

	pub(crate) fn draw(&mut self, gpu_context: &GpuContext) {
		let surface_texture = match self.surface.get_current_texture() {
			wgpu::CurrentSurfaceTexture::Success(texture) => texture,
			wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
			wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
				self.configure_surface(gpu_context);
				texture
			}
			wgpu::CurrentSurfaceTexture::Outdated => {
				self.configure_surface(gpu_context);
				return;
			}
			wgpu::CurrentSurfaceTexture::Validation => {
				unreachable!("No error scope registered, so validation errors will panic")
			}
			wgpu::CurrentSurfaceTexture::Lost => {
				let surface_target =
					unsafe { SurfaceTargetUnsafe::from_window(&self.window).unwrap() };
				self.surface =
					unsafe { gpu_context.instance.create_surface_unsafe(surface_target).unwrap() };
				self.configure_surface(gpu_context);
				return;
			}
		};

		let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
			// Without add_srgb_suffix() the image we will be working with
			// might not be "gamma correct".
			format: Some(self.surface_format.add_srgb_suffix()),
			..Default::default()
		});

		self.renderer.draw(gpu_context, &view);

		self.window.pre_present_notify();
		gpu_context.queue.present(surface_texture);
	}
}
