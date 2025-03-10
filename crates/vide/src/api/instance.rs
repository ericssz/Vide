use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Instance {
  pub matrix: [[f32; 4]; 4],
  pub color: [f32; 4],
  pub radius: f32,
}

impl Instance {
  pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &[
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x4,
          offset: 0,
          shader_location: 5,
        },
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x4,
          offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
          shader_location: 6,
        },
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x4,
          offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
          shader_location: 7,
        },
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x4,
          offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
          shader_location: 8,
        },
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x4,
          offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
          shader_location: 9,
        },
        wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32,
          offset: std::mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
          shader_location: 10,
        },
      ],
    }
  }
}
