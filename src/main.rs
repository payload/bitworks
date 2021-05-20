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
enum Item {
    Red,
    Green,
    Blue,
    Circle,
    Rectangle,
    Star,
    Windmill,
}
default_enum!(Item::Red);

/////////////////////////////////////////////////////////////////////

#[derive(Default, Clone)]
struct BuildingState {
    tag: BuildingTag,
    dir: Dir,

    input_items: Vec<Item>,
    output_items: Vec<Item>,
}

/////////////////////////////////////////////////////////////////////