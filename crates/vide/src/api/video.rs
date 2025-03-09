use std::{collections::VecDeque, sync::Arc, time::Duration};

use crate::{api::color::Color, clip::Clip, io::Export, render::Renderer, rgb8};

#[derive(Debug, Clone, Copy)]
pub struct VideoSettings {
  pub fps: f64,
  pub resolution: (u32, u32),
  pub duration: Duration,
  pub background_color: Color,
}

impl Default for VideoSettings {
  fn default() -> Self {
    Self {
      fps: 60.0,
      resolution: (1920, 1080),
      duration: Duration::from_secs(30),
      background_color: rgb8!(0x17, 0x17, 0x17),
    }
  }
}

pub struct Video {
  #[cfg(feature = "preview")]
  event_loop: winit::event_loop::EventLoop<()>,
  #[cfg(feature = "preview")]
  window: Arc<winit::window::Window>,
  pub renderer: Renderer,
  clips: VecDeque<Box<dyn Clip>>,
  pub settings: VideoSettings,
}

impl Video {
  pub fn new(settings: VideoSettings) -> Self {
    #[cfg(feature = "preview")]
    let (event_loop, window, renderer) = {
      let event_loop = winit::event_loop::EventLoop::new().unwrap();
      let window = Arc::new(
        winit::window::WindowBuilder::new()
          .with_inner_size(winit::dpi::PhysicalSize::new(
            settings.resolution.0,
            settings.resolution.1,
          ))
          .with_resizable(false)
          .build(&event_loop)
          .unwrap(),
      );
      let renderer = Renderer::new(settings, window.clone());

      (event_loop, window, renderer)
    };

    Self {
      #[cfg(feature = "preview")]
      event_loop,
      #[cfg(feature = "preview")]
      window,
      #[cfg(feature = "preview")]
      renderer,
      #[cfg(not(feature = "preview"))]
      renderer: Renderer::new(settings),
      clips: VecDeque::new(),
      settings,
    }
  }

  pub fn render(mut self, exporter: impl Export)
  where
    Self: 'static,
  {
    #[cfg(feature = "preview")]
    self.preview();
    #[cfg(not(feature = "preview"))]
    self.export(exporter);
  }

  #[inline]
  pub fn clips(&mut self) -> &VecDeque<Box<dyn Clip>> {
    &self.clips
  }

  #[inline]
  pub fn clips_mut(&mut self) -> &mut VecDeque<Box<dyn Clip>> {
    &mut self.clips
  }

  #[inline]
  pub fn push_clip(&mut self, clip: impl Clip + 'static) {
    self.clips.push_front(Box::new(clip));
  }

  #[inline]
  pub fn remove_clip(&mut self, index: usize) {
    assert!(index < self.clips.len(), "index {} is out of bounds", index);
    self.clips.remove(index);
  }

  #[cfg(feature = "preview")]
  fn preview(self)
  where
    Self: 'static,
  {
    use crate::render::RenderEvent;

    let Video {
      settings,
      window,
      event_loop,
      mut renderer,
      mut clips,
      ..
    } = self;

    let mut frame = 0u64;
    let _ = event_loop.run(move |event, elwt| match event {
      winit::event::Event::WindowEvent {
        event: winit::event::WindowEvent::CloseRequested,
        ..
      } => elwt.exit(),
      winit::event::Event::WindowEvent {
        event: winit::event::WindowEvent::RedrawRequested,
        ..
      } => {
        let mut events = vec![];
        for clip in clips.iter_mut() {
          let start_frame = clip.start();
          if clip.in_time_frame(frame) {
            events.push(RenderEvent::Clip {
              clip: clip.as_mut(),
              frame: frame - start_frame,
            });
          }
        }
        renderer.render(events);
        frame = (frame + 1) % (settings.duration.as_secs_f64() * settings.fps) as u64;
      }
      winit::event::Event::AboutToWait => {
        window.request_redraw();
      }
      _ => (),
    });
  }

  #[cfg(not(feature = "preview"))]
  fn export(mut self, mut exporter: impl Export) {
    use crate::{clip::IntoFrame, render::RenderEvent};

    exporter.begin(self.settings);

    let total_frames = self.settings.duration.into_frame(self.settings.fps);
    for frame in 0..total_frames {
      let mut events = vec![];
      for clip in self.clips.iter_mut() {
        let start_frame = clip.start();
        if clip.in_time_frame(frame) {
          events.push(RenderEvent::Clip {
            clip: clip.as_mut(),
            frame: frame - start_frame,
          });
        }
      }

      let frame_data = self.renderer.render(events).unwrap();
      exporter.push_frame(true, &frame_data);
    }

    exporter.end();
  }
}
