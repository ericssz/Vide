use std::time::Duration;

use vide::prelude::*;

fn main() {
  env_logger::init();

  let mut video = Video::new(VideoSettings {
    duration: Duration::from_secs_f64(5.0),
    ..Default::default()
  });

  let easing_functions = [
    ease::LINEAR,
    ease::IN_QUADRATIC,
    ease::IN_CUBIC,
    ease::IN_QUARTIC,
    ease::IN_QUINTIC,
    ease::IN_EXPONENTIAL,
    ease::OUT_QUADRATIC,
    ease::OUT_CUBIC,
    ease::OUT_QUARTIC,
    ease::OUT_QUINTIC,
    ease::OUT_EXPONENTIAL,
    ease::IN_BACK,
    ease::OUT_BACK,
    ease::IN_OUT_BACK,
  ];

  let rect_size = 600.0 / easing_functions.len() as f32;
  let rect_seperation = 800.0 / easing_functions.len() as f32;
  let pos_left = -820.0 + rect_size * 0.5;
  let pos_right = 820.0 - rect_size * 0.5;
  for (i, &easing) in easing_functions.iter().enumerate() {
    let y_pos = 400.0 - rect_seperation * i as f32 + rect_size * 0.5;
    video.push_clip(
      Rect::builder()
        .position(
          Animated::builder()
            .keyframe(Abs(0.0), ease::LINEAR, (pos_left, y_pos))
            .keyframe(Abs(2.0), easing, (pos_right, y_pos))
            .hold(0.5)
            .keyframe(Abs(4.5), easing, (pos_left, y_pos))
            .build(),
        )
        .size((rect_size, rect_size))
        .color(rgb8!(0xda, 0x00, 0x37))
        .rounded(0.2) // 20%
        .timing(0.0..5.0)
        .build(),
    );
  }

  video.render(vide_export::quick_export::to("output.mp4"));
}
