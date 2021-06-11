#![feature(total_cmp)]
#![feature(drain_filter)]

use bevy::input::system::exit_on_esc_system;
use bevy::math::vec2;
use bevy::prelude::*;

// debug lines
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

mod merger;
use merger::*;

mod game_types;
use game_types::*;

mod systems;
use systems::*;

pub fn belts_example_main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .add_system(exit_on_esc_system.system())
        .add_system(belt_advance_items_system.system())
        .add_system(random_item_generator_system.system())
        .add_system(null_sink_system.system())
        .add_system(merger_system.system())
        .add_system(debug_draw_item_things_system.system())
        .add_system(debug_draw_belt_system.system())
        .add_system(debug_belt_path_place_random_items_system.system())
        .add_startup_system(setup.system());
    app.run();
}

fn setup(mut cmds: Commands) {
    let cmds = &mut cmds;
    cmds.spawn_bundle(camera());
    cmds.spawn_bundle(belt1());

    let belt2_sink = cmds.spawn_bundle(item_sink(vec2(30.0, -20.0))).id();
    cmds.spawn_bundle(belt2(belt2_sink));

    let belt3_sink = cmds.spawn_bundle(item_sink(vec2(30.0, -30.0))).id();
    let belt3 = cmds.spawn_bundle(belt3(belt3_sink)).id();
    cmds.spawn_bundle(item_generator(belt3, vec2(-30.0, -30.0), 0.0));

    {
        let in1 = (ItemInput::new(1), vec2(0.0, -40.0)).spawn(cmds);
        let in2 = (ItemInput::new(1), vec2(0.0, -45.0)).spawn(cmds);
        (NullSink::new(&[in1]),).spawn(cmds);

        let belt1 = belt(-30, -40, in1).spawn(cmds);
        let belt2 = belt(-30, -45, in2).spawn(cmds);

        let merge1 = (ItemInput::new(1), vec2(-35.0, -40.0)).spawn(cmds);
        let merge2 = (ItemInput::new(1), vec2(-35.0, -45.0)).spawn(cmds);

        (Merger {
            cooldown: 0.0,
            next_time: 0.0,
            items_per_step: 1,
            input_cursor: 0,
            output_cursor: 0,
            inputs: vec![merge1, merge2],
            outputs: vec![belt1, belt2],
        },)
            .spawn(cmds);

        let belt_merge1 = belt(-65, -40, merge1).spawn(cmds);
        let belt_merge2 = belt(-65, -45, merge2).spawn(cmds);

        item_generator(belt_merge1, vec2(-65.0, -40.0), 0.5).spawn(cmds);
        item_generator(belt_merge2, vec2(-65.0, -45.0), 0.0).spawn(cmds);
    }
}

trait SpawnBundle {
    fn spawn(self, cmds: &mut Commands) -> Entity;
}

impl<T: Bundle> SpawnBundle for T {
    fn spawn(self, cmds: &mut Commands) -> Entity {
        cmds.spawn_bundle(self).id()
    }
}

fn camera() -> impl Bundle {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.translation.z = 100.0;
    camera.orthographic_projection.scale = 0.25;
    camera
}

fn belt1() -> impl Bundle {
    (Belt {
        segments: vec![
            BeltSegment::straight(-30, 30, 0, 30),
            BeltSegment::straight(0, 30, 30, 50),
        ],
        items: vec![BeltItem::red(0.0), BeltItem::green(30.0)],
        output: None,
    },)
}

fn belt2(output: Entity) -> impl Bundle {
    (Belt {
        segments: vec![
            BeltSegment::straight(-30, 10, 0, 0),
            BeltSegment::straight(0, 0, 30, -20),
        ],
        items: vec![BeltItem::red(0.0), BeltItem::green(30.0)],
        output: Some(output),
    },)
}

fn belt3(output: Entity) -> impl Bundle {
    (Belt {
        segments: vec![
            BeltSegment::straight(-30, -30, 0, -30),
            BeltSegment::straight(0, -30, 30, -30),
        ],
        items: vec![BeltItem::red(0.0), BeltItem::green(30.0)],
        output: Some(output),
    },)
}

fn belt(x: i32, y: i32, output: Entity) -> impl Bundle {
    (Belt {
        segments: vec![BeltSegment::straight(x, y, x + 30, y)],
        items: vec![],
        output: Some(output),
    },)
}

fn item_sink(pos: Vec2) -> impl Bundle {
    (
        pos,
        ItemInput::new(2),
    )
}

fn item_generator(belt: Entity, pos: Vec2, cooldown: f32) -> impl Bundle {
    (
        pos,
        RandomItemGenerator {
            cooldown,
            next_time: 0.0,
            output: Some(belt),
        },
    )
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

struct NullSink {
    inputs: Vec<Entity>,
}

impl NullSink {
    fn new(inputs: &[Entity]) -> Self {
        Self {
            inputs: inputs.into(),
        }
    }
}

fn null_sink_system(mut sinks: Query<&mut NullSink>, mut inputs: Query<&mut ItemInput>) {
    for mut sink in sinks.iter_mut() {
        sink.inputs.drain_filter(|entity| {
            if let Ok(mut input) = inputs.get_mut(*entity) {
                input.clear_items();
                false
            } else {
                true
            }
        });
    }
}

///////////////////////////////////////////////////////////////////////////////

struct RandomItemGenerator {
    next_time: f64,
    cooldown: f32,
    output: Option<Entity>,
}

fn random_item_generator_system(
    mut generators: Query<&mut RandomItemGenerator>,
    mut belts: Query<&mut Belt>,
    time: Res<Time>,
) {
    let time = time.seconds_since_startup();

    for mut generator in generators.iter_mut() {
        if generator.next_time <= time {
            if let Some(output) = generator.output {
                if let Ok(mut belt) = belts.get_mut(output) {
                    let gen_item = BeltItem::new(0.0, Item::random());

                    if belt.is_space(&gen_item) {
                        belt.add_item(gen_item);
                        generator.next_time = time + generator.cooldown as f64;
                    }
                }
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

type Pos = Vec2;

///////////////////////////////////////////////////////////////////////////////
