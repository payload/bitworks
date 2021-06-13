#![feature(total_cmp)]
#![feature(drain_filter)]

pub use bevy::input::system::exit_on_esc_system;
pub use bevy::math::vec2;
pub use bevy::prelude::*;

pub use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

pub use bevy_prototype_lyon::plugin::ShapePlugin as LyonPlugin;

mod merger;
pub use merger::*;

mod game_types;
pub use game_types::*;

mod systems;
pub use systems::*;

mod extension_traits;
pub use extension_traits::*;

pub struct BeltPlugin;
impl Plugin for BeltPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(belt_advance_items_system.system())
            .add_system(random_item_generator_system.system())
            .add_system(null_sink_system.system())
            .add_system(merger_system.system())
            .add_system_to_stage(CoreStage::PreUpdate, belt_input_system.system());
    }
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(debug_draw_item_things_system.system())
            .add_system(debug_draw_belt_system.system())
            .add_system(debug_belt_path_place_random_items_system.system());
    }
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(MapCache::default())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                map_pos_apply_transform_system.system(),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                map_cache_system.system().label("map_cache"),
            );
    }
}

pub fn nice_camera() -> impl Bundle {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.translation.z = 100.0;
    camera.orthographic_projection.scale = 0.25;
    camera
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
    if trigger.just_pressed(KeyCode::Space) {
        for mut belt in belts.iter_mut() {
            let item = BeltItem::new(belt.total_length() * fastrand::f32(), Item::random());
            belt.add_item(item);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

type Pos = Vec2;

///////////////////////////////////////////////////////////////////////////////
