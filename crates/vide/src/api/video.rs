use std::{collections::VecDeque, time::Duration};

use crate::{api::color::Color, app::App, clip::Clip, io::Export, rgb8};

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
  #[cfg(not(feature = "preview"))]
  pub renderer: Renderer,
  clips: VecDeque<Box<dyn Clip>>,
  pub settings: VideoSettings,
}

impl Video {
  pub fn new(settings: VideoSettings) -> Self {
    #[cfg(feature = "preview")]
    {
      Self {
        clips: VecDeque::new(),
        settings,
      }
    }

    #[cfg(not(feature = "preview"))]
    Self {
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
    let (event_loop, mut app) = App::new(self.settings, self.clips);
    event_loop.run_app(&mut app).unwrap();
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
