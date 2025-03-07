use std::sync::MutexGuard;

use super::{
  animation::AnimatedProperty, color::Color, instance::Instance, instanced_mesh::InstancedMesh,
  mesh::Vertex, shader::Shader, transform::OPENGL_TO_WGPU_MATRIX,
};
use crate::{
  effect::{Effect, EffectBackend},
  register_effect,
};

register_effect!(RectBackend, Rect);

pub struct Rect {
  pub position: AnimatedProperty<(f32, f32)>,
  pub size: AnimatedProperty<(f32, f32)>,
  pub color: AnimatedProperty<Color>,
}

pub struct RectBackend {
  mesh: InstancedMesh,
  instances: Vec<Instance>,
}

impl EffectBackend for RectBackend {
  type Instance = Rect;

  fn push(&mut self, instance: &Self::Instance, frame: u64) {
    let position = instance.position.evaluate(frame);
    let size = instance.size.evaluate(frame);
    let color = instance.color.evaluate(frame);

    self.instances.push(Instance {
      matrix: (cgmath::Matrix4::from_translation(cgmath::Vector3::new(
        position.0, position.1, 0.0,
      )) * cgmath::Matrix4::from_nonuniform_scale(size.0, size.1, 1.0)
        * OPENGL_TO_WGPU_MATRIX)
        .into(),
      color: color.into(),
    });
  }

  fn render<'a>(
    &'a mut self,
    pass: MutexGuard<wgpu::RenderPass<'a>>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
  ) {
    self
      .mesh
      .render(pass, device, queue, self.instances.drain(..).collect());
  }
}

impl Effect for RectBackend {
  fn new(renderer: &mut crate::render::Renderer) -> Self {
    let shader = Shader::new(renderer, include_str!("rect.wgsl").into());
    let mesh = InstancedMesh::new(
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

    Self {
      mesh,
      instances: Vec::new(),
    }
  }
}
