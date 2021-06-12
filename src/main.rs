use bevy::{math::vec3, prelude::*};
use bitworks::*;

fn main() {
    belts_example_app().run();
}

pub fn belts_example_app() -> AppBuilder {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .add_plugin(BeltPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(LyonPlugin)
        .add_system(exit_on_esc_system.system())
        .add_system(simple_spawner_system.system())
        .add_startup_system(setup.system());
    app
}

fn setup(mut cmds: Commands) {
    let cmds = &mut cmds;

    cmds.spawn_bundle(nice_camera());

    use CompassDir::*;
    for simple in vec![
        Simple::ItemGenerator(map_pos(-2, 0), E),
        Simple::Belt(map_pos(-1, 0), W, E),
        Simple::Belt(map_pos(0, 0), W, E),
        Simple::Belt(map_pos(1, 0), W, E),
        Simple::NullSink(map_pos(2, 0), W),
    ] {
        cmds.spawn_bundle((simple,));
    }
}

#[derive(Debug, Clone)]
enum Simple {
    /// pos, out direction
    ItemGenerator(MapPos, CompassDir),
    /// pos, in direction, out direction
    Belt(MapPos, CompassDir, CompassDir),
    /// pos, in direction
    NullSink(MapPos, CompassDir),
}

fn simple_spawner_system(simples: Query<(Entity, &Simple), Added<Simple>>, mut cmds: Commands) {
    for (entity, simple) in simples.iter() {
        let mut cmds = cmds.entity(entity);
        cmds.remove::<Simple>();

        match simple {
            Simple::ItemGenerator(pos, _out_dir) => {
                cmds.insert(pos.clone())
                    .insert(RandomItemGenerator {
                        cooldown: 1.0,
                        next_time: 1.0,
                        output: None,
                    })
                    .insert_bundle(lyon().polygon(6, 16.0).outlined(
                        Color::TEAL,
                        Color::BLACK,
                        4.0,
                    ));
            }
            Simple::Belt(pos, in_dir, out_dir) => {
                let pos_vec = pos.vec2();
                let in_vec = 0.5 * in_dir.vec2();
                let out_vec = 0.5 * out_dir.vec2();

                cmds.insert(pos.clone())
                    .insert(BeltSegment {
                        start: 32.0 * vec3(pos_vec.x + in_vec.x, pos_vec.y + in_vec.y, 0.0),
                        end: 32.0 * vec3(pos_vec.x + out_vec.x, pos_vec.y + out_vec.y, 0.0),
                    })
                    .insert_bundle(lyon().polygon(4, 16.0).outlined(
                        Color::GRAY,
                        Color::BLACK,
                        4.0,
                    ));
            }
            Simple::NullSink(pos, _in_dir) => {
                cmds.insert(pos.clone())
                    .insert(NullSink::new(&[]))
                    .insert_bundle(lyon().circle(16.0).outlined(Color::RED, Color::BLACK, 4.0));
            }
        }
    }
}
