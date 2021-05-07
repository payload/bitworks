#![feature(pub_macro_rules)]

use bevy::prelude::*;

mod buildings;
mod components;
mod core;
mod systems;
mod tools;

use crate::core::*;
use buildings::*;
use components::*;
use systems::*;
use tools::*;

// special uses
use bevy::input::system::exit_on_esc_system;

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_system(show_metrics_system.system())
        .add_system(exit_on_esc_system.system())
        .add_system(item_processor_system.system())
        .add_startup_system_to_stage(StartupStage::PostStartup, map_cache_system.system())
        .add_startup_system(setup.system());
    app.run();
}

fn setup(mut cmds: Commands) {
    let to_pos = |x| (x, 0);

    for (x, char) in "    >==x==o   ".chars().enumerate() {
        let pos = to_pos(x);
        match char {
            '>' => {}
            '=' => {}
            'o' => {}
            'x' => {
                cutter4_building(&mut cmds);
            }
            _ => (),
        }
    }

    cmds.insert_resource(MapCache::default());
}
