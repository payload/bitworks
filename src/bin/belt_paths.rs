#![feature(total_cmp)]

use bevy::input::system::exit_on_esc_system;
use bevy::math::{vec2, vec3};
use bevy::prelude::*;

// debug lines
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .add_system(exit_on_esc_system.system())
        .add_system(belt_advance_items_system.system())
        .add_system(random_item_generator_system.system())
        .add_system(debug_draw_item_things_system.system())
        .add_system(debug_draw_belt_system.system())
        .add_system(debug_belt_path_place_random_items_system.system())
        .add_startup_system(setup.system());
    app.run();
}

fn setup(mut cmds: Commands) {
    cmds.spawn_bundle(camera());
    cmds.spawn_bundle(belt1());

    let belt2_sink = cmds.spawn_bundle(item_sink(vec2(30.0, -20.0))).id();
    cmds.spawn_bundle(belt2(belt2_sink));

    let belt3_sink = cmds.spawn_bundle(item_sink(vec2(30.0, -30.0))).id();
    let belt3 = cmds.spawn_bundle(belt3(belt3_sink)).id();
    cmds.spawn_bundle(item_generator(belt3, vec2(-30.0, -30.0)));
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

fn item_sink(pos: Vec2) -> impl Bundle {
    (
        pos,
        ItemInput {
            capacity: 2,
            items: Vec::new(),
        },
    )
}

fn item_generator(belt: Entity, pos: Vec2) -> impl Bundle {
    (
        pos,
        RandomItemGenerator {
            cooldown: 0.0,
            next_time: 0.0,
            output: Some(belt),
        },
    )
}

fn belt_advance_items_system(
    mut belts: Query<&mut Belt>,
    mut item_inputs: Query<&mut ItemInput>,
    time: Res<Time>,
) {
    let time = time.delta_seconds();

    for mut belt in belts.iter_mut() {
        let mut output = belt
            .output
            .and_then(|output| item_inputs.get_mut(output).ok());

        let speed = 10.0;
        let advance = speed * time;

        let total_length = belt.total_length();
        let mut next_stop = if let Some(ref mut output) = output {
            NextStop::Output(output)
        } else {
            NextStop::End(total_length)
        };

        let mut pass_on = 0usize;

        for item in belt.items.iter_mut().rev() {
            let padding = item.padding();
            match next_stop {
                NextStop::End(stop) => {
                    item.pos = stop.min(item.pos + advance);
                    next_stop = NextStop::Item(item.pos - padding);
                }
                NextStop::Item(stop) => {
                    item.pos = (stop - padding).min(item.pos + advance);
                    next_stop = NextStop::Item(item.pos - padding);
                }
                NextStop::Output(ref output) => {
                    if item.pos + advance > total_length {
                        // when item is passed on, item.pos is set to the overflow after total length
                        let space = output.space();
                        if space > 0 {
                            pass_on += 1;
                            item.pos = item.pos + advance - total_length;

                            if space == 1 {
                                next_stop = NextStop::End(total_length);
                            }
                        } else {
                            item.pos = total_length;
                            next_stop = NextStop::Item(item.pos - padding);
                        }
                    } else {
                        item.pos += advance;
                        next_stop = NextStop::Item(item.pos - padding);
                    }
                }
            };
        }

        if pass_on > 0 {
            let mut output = output.expect("only pass on if output exists");
            let split_at = belt.items.len() - pass_on;
            output.items.extend(belt.items.split_off(split_at));
        }
    }
}

enum NextStop<'a> {
    End(f32),
    Item(f32),
    Output(&'a mut ItemInput),
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
        for segment in belt.segments.iter() {
            let normal = (segment.end - segment.start).any_orthogonal_vector().normalize();
            lines.line_colored(segment.start + normal, segment.end + normal, 0.015, Color::BLACK);
        }

        for item in belt.items.iter() {
            let (pos, dir) = belt.location_on_path(item.pos);
            let start = pos - 0.5 * dir;
            let end = pos + 0.5 * dir;
            lines.line_colored(start, end, 0.02, item.item.color());
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

struct Belt {
    segments: Vec<BeltSegment>,
    items: Vec<BeltItem>,
    output: Option<Entity>,
}

impl Belt {
    fn add_item(&mut self, item: BeltItem) {
        let index = self
            .items
            .binary_search_by(|other| other.pos.total_cmp(&item.pos))
            .map_or_else(|i| i, |i| i);
        self.items.insert(index, item);
    }

    /// go through each segment, accumulate segment lengths,
    /// until there is the segment with this pos
    /// and return Vec3 pos with direction
    /// or else return end or zero
    fn location_on_path(&self, pos: f32) -> (Vec3, Vec3) {
        let mut accu = 0.0;

        for segment in self.segments.iter() {
            let diff = segment.end - segment.start;
            let length = diff.length();
            let dir = diff.normalize_or_zero();
            let segment_pos = pos - accu;

            if segment_pos >= 0.0 && segment_pos <= length {
                return (segment.start + dir * segment_pos, dir);
            } else {
                accu += length;
            }
        }

        if let Some(segment) = self.segments.last() {
            let diff = segment.end - segment.start;
            let dir = diff.normalize_or_zero();
            (segment.end, dir)
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        }
    }

    fn total_length(&self) -> f32 {
        self.segments
            .iter()
            .fold(0.0, |acc, seg| acc + seg.start.distance(seg.end))
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BeltItem {
    pos: f32,
    item: Item,
}

impl BeltItem {
    fn new(pos: f32, item: Item) -> Self {
        Self { pos, item }
    }

    fn red(pos: f32) -> Self {
        Self::new(pos, Item::Red)
    }

    fn green(pos: f32) -> Self {
        Self::new(pos, Item::Green)
    }

    fn padding(&self) -> f32 {
        1.0
    }
}

///////////////////////////////////////////////////////////////////////////////

struct BeltSegment {
    start: Vec3,
    end: Vec3,
}

impl BeltSegment {
    fn straight(startx: i32, starty: i32, endx: i32, endy: i32) -> Self {
        Self {
            start: vec3(startx as f32, starty as f32, 0.0),
            end: vec3(endx as f32, endy as f32, 0.0),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
enum Item {
    Red,
    Green,
}

impl Item {
    fn color(&self) -> Color {
        match &self {
            Item::Red => Color::RED,
            Item::Green => Color::GREEN,
        }
    }

    fn random() -> Self {
        use Item::*;
        let items = [Red, Green];
        items[fastrand::usize(0..items.len())]
    }
}

///////////////////////////////////////////////////////////////////////////////

struct ItemInput {
    items: Vec<BeltItem>,
    capacity: usize,
}

impl ItemInput {
    fn space(&self) -> usize {
        self.capacity.saturating_sub(self.items.len())
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

                    if is_space(&gen_item, &belt) {
                        belt.items.insert(0, gen_item);
                        generator.next_time += generator.cooldown as f64;
                    }
                }
            }
        }
    }

    fn is_space(gen_item: &BeltItem, belt: &Belt) -> bool {
        if let Some(item) = belt.items.first() {
            gen_item.padding() <= item.pos - item.padding()
        } else {
            true
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

type Pos = Vec2;

///////////////////////////////////////////////////////////////////////////////
