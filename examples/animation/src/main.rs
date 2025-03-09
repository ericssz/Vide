use std::time::Duration;

use vide::prelude::*;

fn main() {
  env_logger::init();

  let mut video = Video::new(VideoSettings {
    duration: Duration::from_secs_f64(5.0),
    ..Default::default()
  });

  video.push_clip(
    Rect::builder()
      .position((-300.0, 0.0))
      .size((200.0, 150.0))
      .color(
        Animation::new(60.0)
          .keyframe(Abs(0.0), ease::LINEAR, rgba8!(0xda, 0x00, 0x37, 0x00))
          .keyframe(Rel(0.3), ease::OUT_QUADRATIC, rgb8!(0xda, 0x00, 0x37))
          .hold(0.3)
          .keyframe(Rel(0.3), ease::IN_QUADRATIC, rgb8!(0x00, 0xda, 0x37))
          .build(),
      )
      .timing(1.0..5.0)
      .build(),
  );

  video.push_clip(
    Rect::builder()
      .position((0.0, 0.0))
      .size((200.0, 150.0))
      .color(
        Animation::new(60.0)
          .keyframe(Abs(0.0), ease::LINEAR, rgba8!(0xda, 0x00, 0x37, 0x00))
          .keyframe(Rel(0.3), ease::OUT_QUADRATIC, rgb8!(0xda, 0x00, 0x37))
          .hold(0.3)
          .keyframe(Rel(0.3), ease::IN_QUADRATIC, rgb8!(0x00, 0xda, 0x37))
          .build(),
      )
      .timing(1.0..5.0)
      .build(),
  );

  video.push_clip(
    Rect::builder()
      .position((300.0, 0.0))
      .size((200.0, 150.0))
      .color(
        Animation::new(60.0)
          .keyframe(Abs(0.0), ease::LINEAR, rgba8!(0xda, 0x00, 0x37, 0x00))
          .keyframe(Rel(0.3), ease::OUT_QUADRATIC, rgb8!(0xda, 0x00, 0x37))
          .hold(0.3)
          .keyframe(Rel(0.3), ease::IN_QUADRATIC, rgb8!(0x00, 0xda, 0x37))
          .build(),
      )
      .timing(1.0..5.0)
      .build(),
  );

  video.push_clip(
    Rect::builder()
      .position((0.0, 0.0))
      .size(
        Animation::new(60.0)
          .keyframe(Abs(0.0), ease::LINEAR, (0.0, 1080.0))
          .keyframe(Rel(0.9), ease::OUT_EXPONENTIAL, (1920.0, 1080.0))
          .build(),
      )
      .color(rgb8!(0x00, 0x37, 0xda))
      .timing(1.0..5.0)
      .build(),
  );

  video.render(vide_export::quick_export::to("output.mp4"));
}
