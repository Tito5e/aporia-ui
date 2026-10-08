mod context;

pub use context::GpuContext;

use wgpu::{
	BlendState, Buffer, BufferUsages, Color, ColorTargetState, ColorWrites,
	CommandEncoderDescriptor, Device, Face, FragmentState, FrontFace, LoadOp, MultisampleState,
	Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
	RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
	ShaderModuleDescriptor, ShaderSource, StoreOp, TextureFormat, TextureView, VertexState,
	util::{BufferInitDescriptor, DeviceExt},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 3],
}

impl Vertex {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x3,
				},
			],
		}
	}
}

const VERTICES: &[Vertex] = &[
	Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
	Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
	Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

pub struct Renderer {
	render_pipeline: RenderPipeline,
	temp_buffer: Buffer,
}

impl Renderer {
	pub fn new(device: &Device, format: TextureFormat) -> Self {
		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Temp Vertex Buffer"),
			contents: bytemuck::cast_slice(VERTICES),
			usage: BufferUsages::VERTEX,
		});

		let shader = device.create_shader_module(ShaderModuleDescriptor {
			label: Some("Test Triangle Shader"),
			source: ShaderSource::Wgsl(include_str!("test.wgsl").into()),
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
				buffers: &[Some(Vertex::desc())],
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

		Self { render_pipeline, temp_buffer: vertex_buffer }
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
					ops: Operations { load: LoadOp::Clear(Color::GREEN), store: StoreOp::Store },
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
				multiview_mask: None,
			});

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_vertex_buffer(0, self.temp_buffer.slice(..));
			render_pass.draw(0..3, 0..1);
		}

		gpu.queue.submit([encoder.finish()]);
	}
}
