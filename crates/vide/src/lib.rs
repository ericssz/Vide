// #![warn(missing_docs)]

// TODO: Add docs for these modules

pub mod api;
pub mod clip;
pub mod io;
pub mod render;

pub use cgmath;
pub use paste;

/// Contains everything you need to get started with Vide, just `use
/// vide::prelude::*` and you're set!
pub mod prelude {
  pub use super::{
    api::{
      animation::{ease, AnimatedPropertyBuilder as Animation, KeyframeTiming::*},
      color::*,
      rect::Rect,
      transform::Transform,
      video::*,
    },
    cubic_bezier, lerp, rgb8, rgba8, unanimated,
  };
}
