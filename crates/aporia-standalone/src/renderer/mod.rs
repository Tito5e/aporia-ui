mod context;

use std::borrow::Cow;

pub use context::GpuContext;

use wgpu::{
	BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, Face,
	FragmentState, FrontFace, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
	PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
	RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
	ShaderSource, StoreOp, Surface, SurfaceCapabilities, SurfaceConfiguration, SurfaceTargetUnsafe,
	TextureFormat, TextureView, VertexState,
	rwh::{HasDisplayHandle, HasWindowHandle},
};

pub struct Renderer {
	render_pipeline: RenderPipeline,
}

impl Renderer {
	pub fn new(device: &Device, format: TextureFormat) -> Self {
		let shader = device.create_shader_module(ShaderModuleDescriptor {
			label: Some("Test Triangle Shader"),
			source: ShaderSource::Wgsl(Cow::Borrowed(
				r#"
					@vertex
					fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
						let x = f32(i32(in_vertex_index) - 1);
						let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
						return vec4<f32>(x, y, 0.0, 1.0);
					}

					@fragment
					fn fs_main() -> @location(0) vec4<f32> {
						return vec4<f32>(0.2, 0.6, 1.0, 1.0);
					}
					"#,
			)),
		});

		let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[],
			immediate_size: 0,
		});

		let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: VertexState {
				module: &shader,
				entry_point: Some("vs_main"),
				buffers: &[],
				compilation_options: Default::default(),
			},
			fragment: Some(FragmentState {
				module: &shader,
				entry_point: Some("fs_main"),
				targets: &[Some(ColorTargetState {
					format,
					blend: Some(BlendState::REPLACE),
					write_mask: ColorWrites::ALL,
				})],
				compilation_options: Default::default(),
			}),
			primitive: PrimitiveState {
				topology: PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: FrontFace::Ccw,
				cull_mode: Some(Face::Back),
				polygon_mode: PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None,
			multisample: MultisampleState::default(),
			multiview_mask: None,
			cache: None,
		});

		Self { render_pipeline }
	}

	pub fn draw(&self, gpu: &GpuContext, view: &TextureView) {
		let mut encoder = gpu.device.create_command_encoder(&CommandEncoderDescriptor {
			label: Some("Render Command Encoder"),
		});

		{
			let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
				label: Some("Main Render Pass"),
				color_attachments: &[Some(RenderPassColorAttachment {
					view,
					depth_slice: None,
					resolve_target: None,
					ops: Operations {
						load: LoadOp::Clear(Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
						store: StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
				multiview_mask: None,
			});

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.draw(0..3, 0..1);
		}

		gpu.queue.submit([encoder.finish()]);
	}
}
