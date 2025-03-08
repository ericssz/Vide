use std::sync::MutexGuard;

use super::{
  animation::AnimatedProperty, color::Color, instance::Instance, instanced_mesh::InstancedMesh,
  mesh::Vertex, shader::Shader, transform::OPENGL_TO_WGPU_MATRIX,
};
use crate::{clip::Clip, render::Renderer};

pub struct Rect {
  pub position: AnimatedProperty<(f32, f32)>,
  pub size: AnimatedProperty<(f32, f32)>,
  pub color: AnimatedProperty<Color>,
  pub start: u64,
  pub end: u64,
}

impl Clip for Rect {
  fn start(&self) -> u64 {
    self.start
  }

  fn end(&self, video_end: u64) -> u64 {
    if self.end == u64::MAX {
      video_end
    } else {
      self.end
    }
  }

  fn in_time_frame(&self, frame: u64) -> bool {
    let start_frame = self.start();
    if frame < start_frame {
      return false;
    }

    if self.end == u64::MAX {
      return true;
    }

    frame < self.end
  }

  fn render(
    &mut self,
    renderer: &mut Renderer,
    pass: MutexGuard<wgpu::RenderPass<'_>>,
    frame: u64,
  ) {
    let position = self.position.evaluate(frame);
    let size = self.size.evaluate(frame);
    let color = self.color.evaluate(frame);

    let shader = Shader::new(renderer, include_str!("rect.wgsl").into());
    let mut mesh = InstancedMesh::new(
      renderer,
      vec![
        Vertex {
          position: [-0.5, -0.5],
          uv: [0.0, 1.0],
        },
        Vertex {
          position: [0.5, -0.5],
          uv: [1.0, 1.0],
        },
        Vertex {
          position: [-0.5, 0.5],
          uv: [0.0, 0.0],
        },
        Vertex {
          position: [0.5, 0.5],
          uv: [1.0, 0.0],
        },
      ],
      Some(vec![0, 1, 2, 2, 1, 3]),
      shader,
    );

    let instance = Instance {
      matrix: (cgmath::Matrix4::from_translation(cgmath::Vector3::new(
        position.0, position.1, 0.0,
      )) * cgmath::Matrix4::from_nonuniform_scale(size.0, size.1, 1.0)
        * OPENGL_TO_WGPU_MATRIX)
        .into(),
      color: color.into(),
    };

    mesh.render(
      pass,
      renderer.wgpu_device(),
      renderer.wgpu_queue(),
      vec![instance],
    );
  }
}
