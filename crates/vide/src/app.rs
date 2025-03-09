use std::{collections::VecDeque, sync::Arc};

use crate::{
  clip::Clip,
  prelude::VideoSettings,
  render::{RenderEvent, Renderer},
};

pub struct App {
  window: Option<Arc<winit::window::Window>>,
  settings: VideoSettings,
  renderer: Option<Renderer>,
  frame: u64,
  clips: VecDeque<Box<dyn Clip>>,
}

impl App {
  pub fn new(
    settings: VideoSettings,
    clips: VecDeque<Box<dyn Clip>>,
  ) -> (winit::event_loop::EventLoop<()>, Self) {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    (
      event_loop,
      Self {
        window: None,
        settings,
        renderer: None,
        frame: 0,
        clips,
      },
    )
  }
}

impl winit::application::ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window = Arc::new(
      event_loop
        .create_window(
          winit::window::WindowAttributes::default()
            .with_inner_size(winit::dpi::PhysicalSize::new(
              self.settings.resolution.0,
              self.settings.resolution.1,
            ))
            .with_resizable(false),
        )
        .unwrap(),
    );

    self.renderer = Some(Renderer::new(self.settings, window.clone()));
    self.window = Some(window);
  }

  fn window_event(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    _id: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    match event {
      winit::event::WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      winit::event::WindowEvent::RedrawRequested => {
        if let Some(renderer) = &mut self.renderer {
          let mut events = vec![];
          for clip in self.clips.iter_mut() {
            let start_frame = clip.start();
            if clip.in_time_frame(self.frame) {
              events.push(RenderEvent::Clip {
                clip: clip.as_mut(),
                frame: self.frame - start_frame,
              });
            }
          }
          renderer.render(events);
          self.frame =
            (self.frame + 1) % (self.settings.duration.as_secs_f64() * self.settings.fps) as u64;
        }
        self.window.as_ref().unwrap().request_redraw();
      }
      _ => (),
    }
  }
}
