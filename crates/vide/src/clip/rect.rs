use std::sync::MutexGuard;

use super::Clip;
use crate::{
  api::{
    animation::Animated, color::Color, instance::Instance, mesh::Mesh, shader::Shader,
    transform::OPENGL_TO_WGPU_MATRIX, vertex::Vertex,
  },
  render::Renderer,
  unanimated,
};

pub struct Rect {
  pub position: Animated<(f32, f32)>,
  pub size: Animated<(f32, f32)>,
  pub color: Animated<Color>,
  pub radius: Animated<f32>,
  pub start: f64,
  pub end: f64,
}

impl Rect {
  pub fn builder() -> RectBuilder {
    RectBuilder::default()
  }
}

impl Clip for Rect {
  fn start(&self) -> u64 {
    (self.start * 60.0) as u64
  }

  fn end(&self, video_end: u64) -> u64 {
    if self.end.is_infinite() {
      video_end
    } else {
      (self.end * 60.0) as u64
    }
  }

  fn in_time_frame(&self, frame: u64) -> bool {
    let start_frame = self.start();
    if frame < start_frame {
      return false;
    }

    if self.end.is_infinite() {
      return true;
    }

    frame < (self.end * 60.0) as u64
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
    let radius = self.radius.evaluate(frame);

    let shader = Shader::new(renderer, include_str!("../shaders/rect.wgsl").into());
    let mut mesh = Mesh::new(
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
      radius,
    };

    mesh.render(
      pass,
      renderer.wgpu_device(),
      renderer.wgpu_queue(),
      vec![instance],
    );
  }
}

pub struct RectBuilder {
  position: Option<Animated<(f32, f32)>>,
  size: Option<Animated<(f32, f32)>>,
  color: Option<Animated<Color>>,
  radius: Option<Animated<f32>>,
  start: f64,
  end: f64,
}

impl Default for RectBuilder {
  fn default() -> Self {
    Self {
      position: None,
      size: None,
      color: None,
      radius: None,
      start: 0.0,
      end: f64::INFINITY,
    }
  }
}

impl RectBuilder {
  pub fn position(mut self, position: impl Into<Animated<(f32, f32)>>) -> Self {
    self.position = Some(position.into());
    self
  }

  pub fn size(mut self, size: impl Into<Animated<(f32, f32)>>) -> Self {
    self.size = Some(size.into());
    self
  }

  pub fn color(mut self, color: impl Into<Animated<Color>>) -> Self {
    self.color = Some(color.into());
    self
  }

  pub fn rounded(mut self, radius: impl Into<Animated<f32>>) -> Self {
    self.radius = Some(radius.into());
    self
  }

  pub fn timing(mut self, range: impl Into<std::ops::Range<f64>>) -> Self {
    let range = range.into();
    self.start = range.start;
    self.end = range.end;
    self
  }

  pub fn build(self) -> Rect {
    Rect {
      position: self.position.unwrap_or_else(|| unanimated!((0.0, 0.0))),
      size: self.size.unwrap_or_else(|| unanimated!((100.0, 100.0))),
      color: self.color.unwrap_or_else(|| unanimated!(Color::WHITE)),
      radius: self.radius.unwrap_or_else(|| unanimated!(0.0)),
      start: self.start,
      end: self.end,
    }
  }
}
