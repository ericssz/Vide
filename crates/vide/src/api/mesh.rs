use std::sync::MutexGuard;

use wgpu::util::DeviceExt;

use super::{instance::Instance, shader::Shader, vertex::Vertex};
use crate::render::Renderer;

#[derive(Debug)]
pub struct Mesh {
  _vertices: Vec<Vertex>,
  len_vertices: u32,
  _indices: Option<Vec<u16>>,
  len_indices: u32,
  _shader: Shader,

  vertex_buffer: wgpu::Buffer,
  index_buffer: Option<wgpu::Buffer>,
  instance_buffer: wgpu::Buffer,
  instance_buffer_len: usize,
  pipeline: wgpu::RenderPipeline,
}

impl Mesh {
  pub fn new(
    renderer: &mut Renderer,
    vertices: Vec<Vertex>,
    indices: Option<Vec<u16>>,
    shader: Shader,
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

    let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("Mesh Instance Buffer"),
      size: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[renderer.wgpu_transform_bind_group_layout()],
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader.module,
        entry_point: Some("vs_main"),
        buffers: &[Vertex::desc(), Instance::desc()],
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
      primitive: wgpu::PrimitiveState::default(),
      depth_stencil: Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
      }),
      multisample: wgpu::MultisampleState::default(),
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
      instance_buffer,
      instance_buffer_len: 0,
      pipeline,
    }
  }

  pub fn render(
    &mut self,
    mut render_pass: MutexGuard<wgpu::RenderPass<'_>>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    instances: Vec<Instance>,
  ) {
    if self.instance_buffer_len != instances.len() {
      self.instance_buffer_len = instances.len();
      self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&instances[..]),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      });
    }

    queue.write_buffer(
      &self.instance_buffer,
      0,
      bytemuck::cast_slice(&instances[..]),
    );

    if let Some(index_buffer) = self.index_buffer.as_ref() {
      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
      render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
      render_pass.draw_indexed(0..self.len_indices, 0, 0..(self.instance_buffer_len as u32));
    } else {
      render_pass.set_pipeline(&self.pipeline);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
      render_pass.draw(0..self.len_vertices, 0..(self.instance_buffer_len as u32));
    }
  }
}
