use std::sync::MutexGuard;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::shader::Shader;
use crate::render::Renderer;

#[derive(Default, Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
  pub position: [f32; 2],
  pub uv: [f32; 2],
}

impl Vertex {
  pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x2,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }
}

#[derive(Debug)]
pub struct Mesh {
  _vertices: Vec<Vertex>,
  len_vertices: u32,
  _indices: Option<Vec<u16>>,
  len_indices: u32,
  _shader: Shader,

  vertex_buffer: wgpu::Buffer,
  index_buffer: Option<wgpu::Buffer>,
  pipeline: wgpu::RenderPipeline,
}

impl Mesh {
  pub fn new(
    renderer: &mut Renderer,
    vertices: Vec<Vertex>,
    indices: Option<Vec<u16>>,
    shader: Shader,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
  ) -> Self {
    let device = renderer.wgpu_device();
    let config = renderer.wgpu_config();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&vertices[..]),
      usage: wgpu::BufferUsages::VERTEX,
    });
    let len_vertices = vertices.len() as u32;

    let (index_buffer, len_indices) = if let Some(indices) = indices.as_ref() {
      (
        Some(
          device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices[..]),
            usage: wgpu::BufferUsages::INDEX,
          }),
        ),
        indices.len() as u32,
      )
    } else {
      (None, 0)
    };

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &{
        let mut l = vec![renderer.wgpu_transform_bind_group_layout()];
        l.extend(bind_group_layouts);
        l
      }[..],
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader.module,
        entry_point: Some("vs_main"),
        buffers: &[Vertex::desc()],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader.module,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
      },
      depth_stencil: Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
      cache: None,
    });

    Self {
      _vertices: vertices,
      len_vertices,
      _indices: indices,
      len_indices,
      _shader: shader,
      vertex_buffer,
      index_buffer,
      pipeline,
    }
  }

  pub fn render(&self, mut render_pass: MutexGuard<wgpu::RenderPass<'_>>, queue: &wgpu::Queue) {
    if let Some(index_buffer) = self.index_buffer.as_ref() {
      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
      render_pass.draw_indexed(0..self.len_indices, 0, 0..1);
    } else {
      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw(0..self.len_vertices, 0..1);
    }
  }
}
