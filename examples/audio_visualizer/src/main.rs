use std::{env::args, time::Duration};

use spectrum_analyzer::{
  samples_fft_to_spectrum, scaling::scale_to_zero_to_one, windows::hann_window, FrequencyLimit,
};
use vide::prelude::*;

const BARS: usize = 100;
const BAR_HEIGHT: f32 = 500.0;
const ALL_BARS_WIDTH: f32 = 1000.0;
const BAR_SEPERATION: f32 = 4.0;
const MIN_FREQ: f32 = 20.0;
const MAX_FREQ: f32 = 20000.0;

fn main() {
  env_logger::init();

  let path = args().nth(1).expect("Please provide a path to a .wav file");

  log::info!("Loading audio file");

  let mut reader = hound::WavReader::open(path).expect("Unable to open .wav file");
  let spec = reader.spec();
  let sample_rate = spec.sample_rate;
  let samples = reader
    .samples::<f32>()
    .map(|v| v.unwrap())
    .collect::<Vec<_>>();

  let channel_samples = if spec.channels == 1 {
    vec![samples]
  } else {
    if spec.channels != 2 {
      eprintln!("The visualizer only supports a maximum of 2 audio channels");
      eprintln!("you will lose information");
    }

    let (left, right): (Vec<_>, Vec<_>) =
      samples.chunks(2).map(|chunk| (chunk[0], chunk[1])).unzip();

    vec![left, right]
  };

  let duration = Duration::from_secs_f64(channel_samples[0].len() as f64 / sample_rate as f64);

  let mut video = Video::new(VideoSettings {
    duration,
    resolution: (1920, 1080),
    background_color: rgb8!(0x00, 0x05, 0x05),
    ..Default::default()
  });

  log::info!("Analyzing audio file");

  let bar_x_size = ALL_BARS_WIDTH / BARS as f32 - BAR_SEPERATION;

  let mut builders: Vec<AnimatedBuilder<(f32, f32)>> = Vec::with_capacity(BARS);

  for _ in 0..BARS {
    let mut animation = Animated::builder();
    animation.keyframe(Abs(0.0), ease::LINEAR, (bar_x_size, 0.0));
    builders.push(animation);
  }

  let freq_step = (MAX_FREQ - MIN_FREQ) / (BARS - 1) as f32;
  let samples_per_frame = sample_rate as usize / 60;

  let mut previous_value = [0.0f32; BARS];

  for frame in 0..((duration.as_secs() + 1) * 60) {
    let start = frame as usize * samples_per_frame;
    let end = start + 2048;

    if channel_samples[0].len() < end {
      log::warn!("Lost frame {frame}");
      continue;
    }

    let hann_window = hann_window(&channel_samples[0][start..end]);
    let spectrum = samples_fft_to_spectrum(
      &hann_window,
      sample_rate,
      FrequencyLimit::All,
      Some(&scale_to_zero_to_one),
    )
    .unwrap();

    for bar in 0..BARS {
      let (_, value) = spectrum.freq_val_closest(bar as f32 * freq_step + MIN_FREQ);

      let prev = previous_value[bar];
      let value = (prev * 0.84).max(value.val() * 5000.0);
      previous_value[bar] = value;

      builders[bar].keyframe(
        Abs(frame),
        ease::LINEAR,
        (bar_x_size, (BAR_HEIGHT * (value / 5000.0)).max(2.0)),
      );
    }
  }

  log::info!("Building animations");

  for (i, builder) in builders.into_iter().enumerate() {
    video.push_clip(
      Rect::builder()
        .position((
          (ALL_BARS_WIDTH * -0.5) + (bar_x_size + BAR_SEPERATION) * i as f32,
          0.0,
        ))
        .size(builder.build())
        .color(rgb8!(0x5f, 0xf2, 0xf0))
        .timing(duration.as_secs_f64()..7.0)
        .build(),
    );
  }

  video.push_clip(
    Rect::builder()
      .position((0.0, 0.0))
      .size((1920.0, 1080.0))
      .color(
        Animated::builder()
          .keyframe(Abs(0.0), ease::LINEAR, rgba8!(0x00, 0x00, 0x00, 0x00))
          .keyframe(Abs(0.6), ease::LINEAR, rgba8!(0x00, 0x00, 0x00, 0x66))
          .hold(5.0)
          .keyframe(Rel(0.6), ease::LINEAR, rgba8!(0x00, 0x00, 0x00, 0x00))
          .build(),
      )
      .timing(0.0..7.0)
      .build(),
  );

  video.push_clip(
    Rect::builder()
      .position(
        Animated::builder()
          .keyframe(Abs(0.0), ease::LINEAR, (0.0, -590.0))
          .keyframe(Rel(1.0), ease::IN_OUT_QUINTIC, (0.0, 0.0))
          .build(),
      )
      .size(
        Animated::builder()
          .keyframe(Abs(0.0), ease::LINEAR, (100.0, 100.0))
          .hold(1.0)
          .keyframe(Rel(0.6), ease::IN_OUT_QUINTIC, (500.0, 128.0))
          .hold(3.0)
          .keyframe(Rel(0.6), ease::IN_QUARTIC, (0.0, 164.0))
          .build(),
      )
      .color(rgb8!(0x04, 0x2f, 0x2e))
      .timing(0.0..7.0)
      .build(),
  );

  video.render(vide_export::quick_export::to("output.mp4"));
}
