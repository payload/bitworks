#![feature(pub_macro_rules)]

use bevy::prelude::*;

//mod buildings;
//mod components;
mod core;
mod systems;
// #[macro_use]
mod tools;

use crate::core::*;
//use buildings::*;
//use components::*;
use systems::*;
//use tools::*;

// special uses
use bevy::input::system::exit_on_esc_system;

/////////////////////////////////////////////////////////////////////

macro_rules! default_enum {
    ($T:ident::$V:ident) => {
        impl Default for $T {
            fn default() -> Self {
                Self::$V
            }
        }
    };
}

/////////////////////////////////////////////////////////////////////

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        //.add_system(show_metrics_system.system())
        .add_system(exit_on_esc_system.system())
        //.add_system(item_ejector_system.system())
        //.add_system(item_processor_system.system())
        .add_system(map_cache_system.system())
        .add_system(process_buildings_system.system())
        .add_startup_system_to_stage(StartupStage::PostStartup, map_cache_system.system())
        .add_startup_system(setup.exclusive_system());
    app.run();
}

default_enum!(BuildingTag::None);

#[derive(Clone)]
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
        (Paintcutter, (5, 3), S),
        (Incinerator, (5, 4), N),
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
        self.spawn().insert_bundle((
            "Condenser".to_string(),
            pos,
            BuildingState {
                tag: BuildingTag::Condenser,
                dir,
                ..Default::default()
            },
        ));
    }

    fn belt_bundle(&mut self, pos: Pos, dir: Dir) {
        self.spawn().insert_bundle((
            "Belt".to_string(),
            pos,
            BuildingState {
                tag: BuildingTag::Belt,
                dir,
                ..Default::default()
            },
        ));
    }

    fn paintcutter_bundle(&mut self, pos: Pos, dir: Dir) {
        self.spawn().insert_bundle((
            "PAintcutter".to_string(),
            pos,
            BuildingState {
                tag: BuildingTag::Paintcutter,
                dir,
                ..Default::default()
            },
        ));
    }

    fn incinerator_bundle(&mut self, pos: Pos, dir: Dir) {
        self.spawn().insert_bundle((
            "Incinerator".to_string(),
            pos,
            BuildingState {
                tag: BuildingTag::Incinerator,
                dir,
                ..Default::default()
            },
        ));
    }
}

/////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(Clone)]
enum Color {
    Gray,
    Red,
    Green,
    Blue,
}

#[allow(dead_code)]
#[derive(Clone)]
enum Shape {
    Circle,
    Rectangle,
    Star,
    Windmill,
}

#[allow(dead_code)]
#[derive(Clone)]
struct Piece(Color, Shape);

#[allow(dead_code)]
#[derive(Clone)]
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

    input_items: Vec<Item>,
    output_items: Vec<Item>,
}

/////////////////////////////////////////////////////////////////////

fn process_buildings_system(mut building: Query<(Entity, &Pos, &mut BuildingState)>, map: Res<MapCache>) {
    for (_, _, mut my) in building.iter_mut() {
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
                my.input_items.clear();
            }
        }
    }

    let mut output = Vec::new();

    for (me, pos, mut my) in building.iter_mut() {
        if let Some(you) = map._at(pos) {
            if you != me {
                output.push((you, my.output_items.drain(0..).collect::<Vec<_>>()));
            }
        }
    }

    for (you, input) in output {
        if let Ok((_, _, mut your)) = building.get_mut(you) {
            your.input_items.extend(input);
        }
    }
}

/////////////////////////////////////////////////////////////////////
