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
        .add_system_to_stage(
            CoreStage::PreUpdate,
            input_output_hookup_system2
                .system()
                .after("map_cache")
                .after("io_hookup")
                .label("io_hookup2"),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            output_item_stuff_hookup_system2
                .system()
                .after("io_hookup2"),
        )
        .add_startup_system(setup.system());
    app
}

fn setup(mut cmds: Commands) {
    let cmds = &mut cmds;

    cmds.spawn_bundle(nice_camera());

    use CompassDir::*;
    for simple in vec![
        Simple::ItemGenerator(map_pos(-3, 2), E),
        Simple::Belt(map_pos(-2, 2), W, E),
        Simple::Belt(map_pos(-1, 2), W, E),
        Simple::ItemGenerator(map_pos(-3, 0), E),
        Simple::Belt(map_pos(-2, 0), W, N),
        Simple::Belt(map_pos(-2, 1), S, E),
        Simple::Belt(map_pos(1, 2), W, E),
        Simple::Belt(map_pos(1, 1), W, E),
        Simple::NullSink(map_pos(2, 2), W),
        Simple::Merger2x2(map_pos(0, 2), E),
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
    // pos of cell 1, output direction, pos of cell 2 is right of output direction
    Merger2x2(MapPos, CompassDir),
}

fn simple_spawner_system(simples: Query<(Entity, &Simple), Added<Simple>>, mut cmds: Commands) {
    for (entity, simple) in simples.iter() {
        cmds.entity(entity).remove::<Simple>();

        println!("spawn {:?} as {:?}", entity, simple);

        match simple {
            Simple::ItemGenerator(pos, out_dir) => {
                cmds.entity(entity)
                    .insert(*pos)
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

                cmds.entity(entity)
                    .insert(*pos)
                    .insert(Belt {
                        segments: vec![segment],
                        items: vec![],
                        output: None,
                    })
                    .insert(ItemInput::new(2))
                    .insert(SingleInput(map_pos(0, 0), *in_dir))
                    .insert(SingleOutput(map_pos(0, 0), *out_dir, None))
                    .insert_bundle(lyon().polygon(4, 16.0).outlined(
                        Color::GRAY,
                        Color::BLACK,
                        4.0,
                    ));
            }
            Simple::NullSink(pos, in_dir) => {
                cmds.entity(entity)
                    .insert(pos.clone())
                    .insert(NullSink::new(&[entity]))
                    .insert(ItemInput::new(2))
                    .insert(SingleInput(map_pos(0, 0), *in_dir))
                    .insert_bundle(lyon().circle(16.0).outlined(Color::RED, Color::BLACK, 4.0));
            }
            Simple::Merger2x2(pos1, out_dir) => {
                let pos1 = *pos1;
                let out_dir = *out_dir;
                let in_dir = out_dir.opposite();
                let pos2 = pos1.step(out_dir.right());
                let right = map_pos(0, 0).step(out_dir.right());

                let in1 = cmds
                    .spawn()
                    .insert(pos1)
                    .insert(ItemInput::new(2))
                    .insert(SingleInput(map_pos(0, 0), in_dir))
                    .id();
                let in2 = cmds
                    .spawn()
                    .insert(pos2)
                    .insert(ItemInput::new(2))
                    .insert(SingleInput(right, in_dir))
                    .id();
                cmds.entity(entity)
                    .insert(pos1)
                    .insert(Transform::default())
                    .insert(GlobalTransform::default())
                    .insert(Merger {
                        cooldown: 0.0,
                        next_time: 0.0,
                        items_per_step: 1,
                        input_cursor: 0,
                        output_cursor: 0,
                        inputs: vec![in1, in2],
                        outputs: vec![],
                    })
                    .insert(MultipleOutputs::new(&[
                        (map_pos(0, 0), out_dir),
                        (map_pos(0, -1), out_dir),
                    ]))
                    .with_children(|child| {
                        child
                            .spawn()
                            .insert_bundle(lyon().rectangle(32.0, 64.0).outlined_pos(
                                Color::DARK_GRAY,
                                Color::BLACK,
                                4.0,
                                vec2(-16.0, 16.0),
                            ));
                    });
            }
        }
    }
}

struct Scream;

fn input_output_hookup_system(
    inputs: Query<(&MapPos, &SingleInput)>,
    mut outputs: Query<(Entity, &MapPos, &mut SingleOutput)>,
    map: Res<MapCache>,
    scream: Query<&Scream>,
    mut cmds: Commands,
) {
    for (o_entity, pos, mut output) in outputs.iter_mut() {
        if output.2 == None {
            let SingleOutput(o_pos, o_dir, _) = &*output;
            let other_pos = (*pos + *o_pos).step(*o_dir);

            if let Some(input_entity) = map.at(&other_pos) {
                if let Some(input) = inputs.get_component::<SingleInput>(input_entity).ok() {
                    let SingleInput(_, i_dir) = input;

                    if *i_dir == o_dir.opposite() {
                        output.2 = Some(input_entity);

                        println!("hook up {:?} to {:?}", o_entity, input_entity);
                    } else {
                        if scream.get(o_entity).is_err() {
                            eprintln!(
                                "check {:?} wrong directions {:?} to {:?}",
                                o_entity, o_dir, i_dir
                            );
                            cmds.entity(o_entity).insert(Scream);
                        }
                    }
                } else {
                    if scream.get(o_entity).is_err() {
                        eprintln!("check {:?} no single input at {:?}", o_entity, other_pos);
                        cmds.entity(o_entity).insert(Scream);
                    }
                }
            } else {
                if scream.get(o_entity).is_err() {
                    eprintln!("check {:?} none at pos {:?}", o_entity, other_pos);
                    cmds.entity(o_entity).insert(Scream);
                }
            }
        }
    }
}

fn output_item_stuff_hookup_system(
    mut entities: Query<
        (
            Entity,
            &SingleOutput,
            Option<&mut RandomItemGenerator>,
            Option<&mut Belt>,
        ),
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

struct MultipleOutputs {
    outputs: Vec<SingleOutput>,
}

impl MultipleOutputs {
    fn new(entries: &[(MapPos, CompassDir)]) -> Self {
        Self {
            outputs: entries
                .iter()
                .map(|(pos, dir)| SingleOutput(*pos, *dir, None))
                .collect(),
        }
    }
}

fn input_output_hookup_system2(
    inputs: Query<(&MapPos, &SingleInput)>,
    mut outputs: Query<(Entity, &MapPos, &mut MultipleOutputs)>,
    map: Res<MapCache>,
    scream: Query<&Scream>,
    mut cmds: Commands,
) {
    for (o_entity, pos, mut outputs) in outputs.iter_mut() {
        for output in outputs.outputs.iter() {
            if output.2 == None {
                let SingleOutput(o_pos, o_dir, _) = &*output;
                let other_pos = (*pos + *o_pos).step(*o_dir);

                if let Some(input_entity) = map.at(&other_pos) {
                    if let Some(input) = inputs.get_component::<SingleInput>(input_entity).ok() {
                        let SingleInput(_, i_dir) = input;

                        if *i_dir == o_dir.opposite() {
                            output.2 = Some(input_entity);

                            println!("hook up {:?} to {:?}", o_entity, input_entity);
                        } else {
                            if scream.get(o_entity).is_err() {
                                eprintln!(
                                    "check {:?} wrong directions {:?} to {:?}",
                                    o_entity, o_dir, i_dir
                                );
                                cmds.entity(o_entity).insert(Scream);
                            }
                        }
                    } else {
                        if scream.get(o_entity).is_err() {
                            eprintln!("check {:?} no single input at {:?}", o_entity, other_pos);
                            cmds.entity(o_entity).insert(Scream);
                        }
                    }
                } else {
                    if scream.get(o_entity).is_err() {
                        eprintln!("check {:?} none at pos {:?}", o_entity, other_pos);
                        cmds.entity(o_entity).insert(Scream);
                    }
                }
            }
        }
    }
}

fn output_item_stuff_hookup_system2(
    mut entities: Query<
        ((Entity, &MultipleOutputs), (Option<&mut Merger>,)),
        Changed<MultipleOutputs>,
    >,
) {
    for ((entity, outputs), it) in entities.iter_mut() {
        if let Some(mut merger) = it.0 {
            merger.outputs.resize(outputs.outputs.len(), Entity::new(0));

            if let Some(e) = outputs.outputs[0].2 {
                merger.outputs[0] = e;
            }
            if let Some(e) = outputs.outputs[1].2 {
                merger.outputs[1] = e;
            }

            println!("merger output  {:?} set to {:?}", entity, merger.outputs);
        }
    }
}
