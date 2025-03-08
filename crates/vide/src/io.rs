use crate::api::video::VideoSettings;

pub trait Export {
  fn begin(&mut self, settings: VideoSettings);
  /// `frame` contains Rgba8UnormSrgb data as bytes (RGBA8)
  fn push_frame(&mut self, keyframe: bool, frame: &[u8]);
  fn end(self);
}
