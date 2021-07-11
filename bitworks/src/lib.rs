#![feature(total_cmp)]
#![feature(drain_filter)]

pub use bevy::math::{vec2, vec3};
pub use bevy::prelude::*;

pub use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

mod merger;
pub use merger::*;

mod systems;
pub use systems::*;

mod extension_traits;
pub use extension_traits::*;

mod assets;
pub use assets::*;

mod config;
pub use config::*;

mod camera;
pub use camera::*;

mod stuff;
pub use stuff::*;
