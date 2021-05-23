#![feature(pub_macro_rules)]

use bevy::prelude::*;
use bevy::render::color::Color as BevyColor;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use bevy_prototype_lyon::prelude::*;

//mod buildings;
//mod components;
mod core;
mod systems;
// #[macro_use]
mod lyon_ext;
mod tools;

use crate::core::*;
//use buildings::*;
//use components::*;
use systems::*;
//use tools::*;
use lyon_ext::*;

// special uses
use bevy::input::system::exit_on_esc_system;

/////////////////////////////////////////////////////////////////////

macro_rules! impl_default {
    ($T:ident$($V:tt)*) => {
        impl Default for $T {
            fn default() -> Self {
                Self$($V)*
            }
        }
    };
}

/////////////////////////////////////////////////////////////////////

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(DebugLinesPlugin)
        //
        //.add_system(show_metrics_system.system())
        .add_system(exit_on_esc_system.system())
        //.add_system(item_ejector_system.system())
        //.add_system(item_processor_system.system())
        .add_system(map_cache_system.system())
        .add_system(process_buildings_system.system().label("process"))
        .add_system(
            sync_pos_with_transform
                .system()
                .label("sync_pos")
                .after("process"),
        )
        .add_system(debug_render_items.system().after("sync_pos"))
        .add_system_to_stage(CoreStage::PostUpdate, debug_building_output_system.system())
        .add_startup_system_to_stage(StartupStage::PostStartup, map_cache_system.system())
        .add_startup_system(setup.exclusive_system());
    app.run();
}

impl_default!(BuildingTag::None);

#[derive(Clone, Debug)]
enum BuildingTag {
    None,
    Condenser,
    Belt,
    Paintcutter,
    Incinerator,
}

fn setup(world: &mut World) {
    use BuildingTag::*;
    use Dir::*;

    let buildings = [
        (Condenser, (3, 3), E),
        (Belt, (4, 3), E),
        (Belt, (5, 3), E),
        (Belt, (6, 3), E),
        //(Paintcutter, (5, 3), S),
        (Incinerator, (7, 3), E),
    ];

    for (building, pos, dir) in &buildings {
        let pos = Pos(pos.0, pos.1);
        match building {
            None => {}
            Condenser => world.condenser_bundle(pos, *dir),
            Belt => world.belt_bundle(pos, *dir),
            Paintcutter => world.paintcutter_bundle(pos, *dir),
            Incinerator => world.incinerator_bundle(pos, *dir),
        }
    }

    world.insert_resource(MapCache::default());

    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform.translation.z = 5.0;
    world.spawn().insert_bundle(camera);

    // TODO: load png for item
    // mut materials: ResMut<Assets<ColorMaterial>>,
    //     let sprite_handle = materials.add(assets.load("branding/icon.png").into());
    // and spawn an entity with sprite bundle for each item
}

/////////////////////////////////////////////////////////////////////

trait WorldExt {
    fn condenser_bundle(&mut self, pos: Pos, dir: Dir);
    fn belt_bundle(&mut self, pos: Pos, dir: Dir);
    fn paintcutter_bundle(&mut self, pos: Pos, dir: Dir);
    fn incinerator_bundle(&mut self, pos: Pos, dir: Dir);
}

impl WorldExt for World {
    fn condenser_bundle(&mut self, pos: Pos, dir: Dir) {
        self.spawn()
            .insert_bundle((
                "Condenser".to_string(),
                pos,
                BuildingState {
                    tag: BuildingTag::Condenser,
                    dir,
                    cooldown: 1.0,
                    ..Default::default()
                },
            ))
            .insert_bundle(lyon().polygon(6, 16.0).outlined(
                BevyColor::TEAL,
                BevyColor::BLACK,
                4.0,
            ));
    }

    fn belt_bundle(&mut self, pos: Pos, dir: Dir) {
        self.spawn()
            .insert_bundle((
                "Belt".to_string(),
                pos,
                BuildingState {
                    tag: BuildingTag::Belt,
                    dir,
                    cooldown: 0.5,
                    ..Default::default()
                },
            ))
            .insert_bundle(lyon().polygon(4, 16.0).outlined(
                BevyColor::GRAY,
                BevyColor::BLACK,
                4.0,
            ));
    }

    fn paintcutter_bundle(&mut self, pos: Pos, dir: Dir) {
        self.spawn()
            .insert_bundle((
                "Paintcutter".to_string(),
                pos,
                BuildingState {
                    tag: BuildingTag::Paintcutter,
                    dir,
                    cooldown: 1.0,
                    ..Default::default()
                },
            ))
            .insert_bundle(lyon().rectangle(32.0, 32.0).outlined(
                BevyColor::LIME_GREEN,
                BevyColor::BLACK,
                4.0,
            ));
    }

    fn incinerator_bundle(&mut self, pos: Pos, dir: Dir) {
        self.spawn()
            .insert_bundle((
                "Incinerator".to_string(),
                pos,
                BuildingState {
                    tag: BuildingTag::Incinerator,
                    dir,
                    cooldown: 1.0,
                    input_slots: vec![InputSlot::default()],
                    ..Default::default()
                },
            ))
            .insert_bundle(
                lyon()
                    .circle(16.0)
                    .outlined(BevyColor::RED, BevyColor::BLACK, 4.0),
            );
    }
}

/////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum Color {
    Gray,
    Red,
    Green,
    Blue,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum Shape {
    Circle,
    Rectangle,
    Star,
    Windmill,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Piece(Color, Shape);

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum Item {
    Color(Color),
    Shape(Piece, Piece, Piece, Piece),
}

impl Item {
    fn paint(self, other: Item) -> Option<Item> {
        use Item::*;

        match (self, other) {
            (Color(color), Shape(p1, p2, p3, p4)) | (Shape(p1, p2, p3, p4), Color(color)) => {
                Some(Shape(
                    Piece(color.clone(), p1.1),
                    Piece(color.clone(), p2.1),
                    Piece(color.clone(), p3.1),
                    Piece(color, p4.1),
                ))
            }
            (Color(_), Color(_)) => None,
            (Shape(_, _, _, _), Shape(_, _, _, _)) => None,
        }
    }

    fn can_paint(&self, other: &Item) -> bool {
        use Item::*;

        match (self, other) {
            (Color(_), Shape(_, _, _, _)) | (Shape(_, _, _, _), Color(_)) => true,
            (Color(_), Color(_)) => false,
            (Shape(_, _, _, _), Shape(_, _, _, _)) => false,
        }
    }
}

/////////////////////////////////////////////////////////////////////

#[derive(Default, Clone)]
struct BuildingState {
    tag: BuildingTag,
    dir: Dir,

    input_slots: Vec<InputSlot>,
    input_items: Vec<Item>,
    output_items: Vec<Item>,

    cooldown: f32,
    cooldown_progress: f32,
}

#[derive(Clone)]
struct InputSlot {
    item: Option<Item>,
    progress: f32,
    ips: f32,
}

impl_default!(InputSlot {
    item: None,
    progress: 0.0,
    ips: 1.0
});

impl InputSlot {
    fn progress(&mut self, seconds: f32) {
        if self.item.is_some() {
            if self.progress < 1.0 {
                self.progress += seconds * self.ips;
            } else {
                self.progress = 1.0;
            }
        } else if self.progress != 0.0 {
            self.progress = 0.0;
        }
    }

    fn take(&mut self) -> Option<Item> {
        self.progress = 0.0;
        self.item.take()
    }

    fn put(&mut self, item: Item) {
        self.progress = 0.0;
        self.item = Some(item);
    }

    fn is_free(&self) -> bool {
        self.item.is_none()
    }
}

/////////////////////////////////////////////////////////////////////

fn process_buildings_system(
    mut building: Query<(Entity, &Pos, &mut BuildingState)>,
    map: Res<MapCache>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        //return;
    }

    for (_, _, mut my) in building.iter_mut() {
        for slot in my.input_slots.iter_mut() {
            slot.progress(time.delta_seconds());
        }
    }

    for (_, _, mut my) in building.iter_mut() {
        my.cooldown_progress -= time.delta_seconds();
        let should_process = my.cooldown_progress <= 0.0;

        if should_process {
            my.cooldown_progress += my.cooldown;

            match my.tag {
                BuildingTag::None => {}
                BuildingTag::Condenser => {
                    my.output_items.push(Item::Shape(
                        Piece(Color::Gray, Shape::Circle),
                        Piece(Color::Gray, Shape::Circle),
                        Piece(Color::Gray, Shape::Circle),
                        Piece(Color::Gray, Shape::Circle),
                    ));
                }
                BuildingTag::Belt => {
                    for item in my.input_items.pop() {
                        my.output_items.push(item);
                    }
                }
                BuildingTag::Paintcutter => {
                    if my.input_items.len() >= 2 {
                        let color = my.input_items.pop().unwrap();
                        let shape = my.input_items.pop().unwrap();
                        if color.can_paint(&shape) {
                            // TODO dont cut yet
                            my.output_items.push(color.paint(shape).unwrap());
                        } else {
                            my.input_items.push(color);
                            my.input_items.push(shape);
                        }
                    }
                }
                BuildingTag::Incinerator => {
                    let slot = my.input_slots.get_mut(0).expect("input slot 0");
                    slot.take();
                }
            }
        }
    }

    let mut output = Vec::new();

    for (me, pos, mut my) in building.iter_mut() {
        if let Some(you) = map.at(&my.dir.pos(pos)) {
            if you != me && my.output_items.len() > 0 {
                output.push((you, my.output_items.drain(0..).collect::<Vec<_>>()));
            }
        }
    }

    for (you, mut input) in output {
        if let Ok((_, _, mut your)) = building.get_mut(you) {
            print!("o");
            if !your.input_slots.is_empty() {
                print!("s");
                let slot = your.input_slots.get_mut(0).expect("just checked");
                if slot.is_free() {
                    print!("p");
                    let item = input.pop().expect("anything must be in output");
                    slot.put(item);
                }
            } else {
                print!("x");
                your.input_items.extend(input);
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////

fn debug_render_items(
    building: Query<(&BuildingState, &GlobalTransform)>,
    mut lines: ResMut<DebugLines>,
) {
    let mut items = Vec::new();

    for (state, transform) in building.iter() {
        print!(".");
        for slot in state.input_slots.iter() {
            print!("s");
            if let Some(item) = &slot.item {
                print!("i");
                let pos = transform.translation;
                let dir = match state.dir {
                    Dir::W => Vec3::new(-1.0, 0.0, 0.0),
                    Dir::E => Vec3::new(1.0, 0.0, 0.0),
                    Dir::N => Vec3::new(0.0, 1.0, 0.0),
                    Dir::S => Vec3::new(0.0, -1.0, 0.0),
                };
                let pos = pos + dir * -16.0 + dir * slot.progress * 16.0;
                items.push((item, pos, dir));
            }
        }
    }

    lines.line_gradient(
        Vec3::ZERO,
        Vec3::new(200.0, -200.0, 0.0),
        0.0,
        BevyColor::BLACK,
        BevyColor::WHITE,
    );

    for (_item, pos, dir) in items {
        let up = if dir.x != 0.0 {
            Vec3::new(0.0, dir.x, 0.0)
        } else {
            Vec3::new(dir.y, 0.0, 0.0)
        } * 4.0;
        let forward = dir * 4.0;
        let down = -up;
        lines.line(pos + up, pos + forward, 0.0);
        lines.line(pos + down, pos + forward, 0.0);
    }
}

/////////////////////////////////////////////////////////////////////

fn debug_building_output_system(building: Query<&BuildingState>, keys: Res<Input<KeyCode>>) {
    if !keys.just_pressed(KeyCode::Space) {
        //return;
    }

    for my in building.iter() {
        println!("{:?} {}", &my.tag, my.input_items.len());
    }

    println!();
}

/////////////////////////////////////////////////////////////////////

fn sync_pos_with_transform(mut query: Query<(&Pos, &mut Transform), Changed<Pos>>) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation.x = pos.0 as f32 * 32.0;
        transform.translation.y = pos.1 as f32 * -32.0;
    }
}

/////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////
