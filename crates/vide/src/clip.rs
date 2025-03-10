pub mod rect;
use std::{sync::MutexGuard, time::Duration};

pub use rect::Rect;

use crate::render::Renderer;

pub trait IntoFrame {
  fn into_frame(self, fps: f64) -> u64;
}

impl IntoFrame for u64 {
  fn into_frame(self, _fps: f64) -> u64 {
    self
  }
}

impl IntoFrame for Duration {
  fn into_frame(self, fps: f64) -> u64 {
    self.as_secs_f64().into_frame(fps)
  }
}

impl IntoFrame for f64 {
  fn into_frame(self, fps: f64) -> u64 {
    (self * fps) as u64
  }
}

pub trait Clip {
  fn start(&self) -> u64;

  fn end(&self, video_end: u64) -> u64;

  fn in_time_frame(&self, frame: u64) -> bool;

  fn render(&mut self, renderer: &mut Renderer, pass: MutexGuard<wgpu::RenderPass<'_>>, frame: u64);
}
