#![feature(total_cmp)]
#![feature(drain_filter)]

pub use bevy::math::vec2;
pub use bevy::prelude::*;

pub use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

pub use bevy_prototype_lyon::plugin::ShapePlugin as LyonPlugin;

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

pub struct BeltPlugin;
impl Plugin for BeltPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::PreUpdate, belt_input_system.system())
            .add_system_set(
                SystemSet::on_update(AppState::GameRunning)
                    .with_system(belt_advance_items_system.system())
                    .with_system(null_sink_system.system())
                    .with_system(random_item_generator_system.system())
                    .with_system(merger_system.system()),
            );
    }
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(MapCache::default())
            .add_system_to_stage(CoreStage::First, map_pos_apply_transform_system.system())
            .add_system_to_stage(CoreStage::First, map_cache_system.system());
    }
}

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(load_belt_atlas.system())
            .add_startup_system(load_item_texture.system());
    }
}

///////////////////////////////////////////////////////////////////////////////

type Pos = Vec2;

///////////////////////////////////////////////////////////////////////////////
