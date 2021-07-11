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

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(debug_draw_item_things_system.system())
            .add_system(debug_belt_path_place_random_items_system.system());

        if std::env::var("BIT_DEBUG_DRAW_BELT").map_or(false, |s| !s.is_empty()) {
            app.add_system(debug_draw_belt_system.system());
        }
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

fn debug_draw_item_things_system(
    mut lines: ResMut<DebugLines>,
    things: Query<(Entity, &Pos), Or<(With<RandomItemGenerator>, With<ItemInput>)>>,
) {
    for (_it, pos) in things.iter() {
        let pos = pos.extend(0.0);
        let x = Vec3::X;
        let y = Vec3::Y;
        lines.line_colored(pos + y, pos + x, 0.0, Color::WHITE);
        lines.line_colored(pos + x, pos - y, 0.0, Color::WHITE);
        lines.line_colored(pos - y, pos - x, 0.0, Color::WHITE);
        lines.line_colored(pos - x, pos + y, 0.0, Color::WHITE);
    }
}

fn debug_draw_belt_system(mut lines: ResMut<DebugLines>, belts: Query<&Belt>) {
    // draw belt segments
    // draw belt items
    // NOTE that debug lines don't draw well over each other, but increase duration
    //      usually overdraws other lines
    for belt in belts.iter() {
        for segment in belt.segments() {
            let normal = (segment.end - segment.start)
                .any_orthogonal_vector()
                .normalize();
            lines.line_colored(
                segment.start + normal,
                segment.end + normal,
                0.015,
                Color::BLACK,
            );
        }

        for item in belt.items() {
            let (pos, dir) = belt.location_on_path(item.pos);
            let start = pos - 0.5 * dir;
            let end = pos + 0.5 * dir;
            lines.line_colored(start, end, 0.02, item.color());
        }
    }
}

fn debug_belt_path_place_random_items_system(
    trigger: Res<Input<KeyCode>>,
    mut belts: Query<&mut Belt>,
) {
    if trigger.just_pressed(KeyCode::R) {
        println!("debug_belt_path_place_random_items_system");
        for mut belt in belts.iter_mut() {
            let item = BeltItem::new(belt.total_length() * fastrand::f32(), Item::random());
            belt.add_item(item);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

type Pos = Vec2;

///////////////////////////////////////////////////////////////////////////////
