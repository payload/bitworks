use bevy::{math::vec3, prelude::*};
use bitworks::*;

fn main() {
    belts_example_app().run();
}

pub fn belts_example_app() -> AppBuilder {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .add_plugin(DebugPlugin)
        .add_plugin(BeltPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(LyonPlugin)
        .add_system(exit_on_esc_system.system())
        .add_system_to_stage(CoreStage::First, simple_spawner_system.system())
        .add_system_to_stage(
            CoreStage::PreUpdate,
            input_output_hookup_system
                .system()
                .after("map_cache")
                .label("io_hookup"),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            output_item_stuff_hookup_system.system().after("io_hookup"),
        )
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

struct SingleInput(MapPos, CompassDir, Entity);
struct SingleOutput(MapPos, CompassDir, Option<Entity>);

fn simple_spawner_system(simples: Query<(Entity, &Simple), Added<Simple>>, mut cmds: Commands) {
    for (entity, simple) in simples.iter() {
        let mut cmds = cmds.entity(entity);
        cmds.remove::<Simple>();

        println!("spawn {:?} as {:?}", entity, simple);

        match simple {
            Simple::ItemGenerator(pos, out_dir) => {
                cmds.insert(*pos)
                    .insert(RandomItemGenerator {
                        cooldown: 1.0,
                        next_time: 1.0,
                        output: None,
                    })
                    .insert(SingleOutput(map_pos(0, 0), *out_dir, None))
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
                let start = 32.0 * vec3(pos_vec.x + in_vec.x, pos_vec.y + in_vec.y, 0.0);
                let end = 32.0 * vec3(pos_vec.x + out_vec.x, pos_vec.y + out_vec.y, 0.0);
                let segment = BeltSegment { start, end };

                cmds.insert(*pos)
                    .insert(Belt {
                        segments: vec![segment],
                        items: vec![],
                        output: None,
                    })
                    .insert(SingleInput(map_pos(0, 0), *in_dir, entity))
                    .insert(SingleOutput(map_pos(0, 0), *out_dir, None))
                    .insert_bundle(lyon().polygon(4, 16.0).outlined(
                        Color::GRAY,
                        Color::BLACK,
                        4.0,
                    ));
            }
            Simple::NullSink(pos, in_dir) => {
                cmds.insert(pos.clone())
                    .insert(NullSink::new(&[]))
                    .insert(SingleInput(map_pos(0,0), *in_dir, entity))
                    .insert_bundle(lyon().circle(16.0).outlined(Color::RED, Color::BLACK, 4.0));
            }
        }
    }
}

fn input_output_hookup_system(
    inputs: Query<(&MapPos, &SingleInput)>,
    mut outputs: Query<(Entity, &MapPos, &mut SingleOutput)>,
    map: Res<MapCache>,
) {
    for (o_entity, pos, mut output) in outputs.iter_mut() {
        if output.2 == None {
            let SingleOutput(o_pos, o_dir, _) = &*output;
            let other_pos = (*pos + *o_pos).step(*o_dir);

            if let Some(input_entity) = map.at(&other_pos) {
                if let Some(input) = inputs.get_component::<SingleInput>(input_entity).ok() {
                    let SingleInput(_, i_dir, i_entity) = input;

                    if *i_dir == o_dir.opposite() {
                        output.2 = Some(*i_entity);

                        println!("hook up {:?} to {:?}", o_entity, i_entity);
                    } else {
                        eprintln!(
                            "check {:?} wrong directions {:?} to {:?}",
                            o_entity, o_dir, i_dir
                        );
                    }
                } else {
                    eprintln!("check {:?} no single input at {:?}", o_entity, other_pos);
                }
            } else {
                eprintln!("check {:?} none at pos {:?}", o_entity, other_pos);
            }
        }
    }
}

fn output_item_stuff_hookup_system(
    mut entities: Query<
        (Entity, &SingleOutput, Option<&mut RandomItemGenerator>, Option<&mut Belt>),
        Changed<SingleOutput>,
    >,
) {
    for (entity, output, item_gen, belt) in entities.iter_mut() {
        match (item_gen, belt) {
            (Some(mut item_gen), None) => {
                item_gen.output = output.2;
                println!("output  {:?} set to {:?}", entity, item_gen.output);
            }
            (None, Some(mut belt)) => {
                belt.output = output.2;
                println!("output  {:?} set to {:?}", entity, belt.output);
            }
            _ => {}
        }
    }
}
