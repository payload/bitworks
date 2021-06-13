use bevy::{math::vec3, prelude::*, utils::HashSet};

use bevy_prototype_lyon::{entity::ShapeBundle, prelude::ShapeColors};
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
        .add_system_to_stage(CoreStage::Update, draw_belt_system.system())
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
        //
        Simple::ItemGenerator(map_pos(-3, 0), E),
        Simple::Belt(map_pos(-2, 0), W, N),
        Simple::Belt(map_pos(-2, 1), S, E),
        Simple::Belt(map_pos(-1, 1), W, E),
        //
        Simple::Merger2x2(map_pos(0, 2), E),
        //
        Simple::Belt(map_pos(1, 2), W, E),
        Simple::Belt(map_pos(1, 1), W, E),
        Simple::NullSink(map_pos(2, 2), W),
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
                    .insert(output((0, 0), *out_dir))
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
                    .insert(input(map_pos(0, 0), *in_dir))
                    .insert(output((0, 0), *out_dir))
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
                    .insert(input(map_pos(0, 0), *in_dir))
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
                    .insert(input(map_pos(0, 0), in_dir))
                    .id();
                let in2 = cmds
                    .spawn()
                    .insert(pos2)
                    .insert(ItemInput::new(2))
                    .insert(input(right, in_dir))
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
                    .insert(outputs(&[
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

struct MultipleOutputs {
    outputs: Vec<SingleOutput>,
}

impl MultipleOutputs {
    fn new(entries: &[(MapPos, CompassDir)]) -> Self {
        Self {
            outputs: entries
                .iter()
                .map(|(pos, dir)| SingleOutput {
                    pos: *pos,
                    dir: *dir,
                    entity: None,
                })
                .collect(),
        }
    }
}

fn output<P: Into<MapPos>>(pos: P, dir: CompassDir) -> MultipleOutputs {
    MultipleOutputs {
        outputs: vec![SingleOutput {
            pos: pos.into(),
            dir,
            entity: None,
        }],
    }
}

fn outputs(entries: &[(MapPos, CompassDir)]) -> MultipleOutputs {
    MultipleOutputs::new(entries)
}

#[derive(Default)]
struct DebugIOHookupSystem {
    hookup: Vec<(Entity, Entity)>,
    wrong_dir: Vec<(Entity, CompassDir, CompassDir)>,
    no_input: Vec<(Entity, MapPos)>,
    none_at: Vec<(Entity, MapPos)>,
    filter_set: HashSet<Entity>,
}

impl DebugIOHookupSystem {
    fn print(&mut self) {
        let set = &mut self.filter_set;
        for it in self.hookup.iter().filter(|e| set.insert(e.0)) {
            eprintln!("check {:?} hookup {:?}", it.0, it.1);
        }
        for it in self.wrong_dir.iter().filter(|e| set.insert(e.0)) {
            eprintln!("check {:?} wrong dir {:?} to {:?}", it.0, it.1, it.2);
        }
        for it in self.no_input.iter().filter(|e| set.insert(e.0)) {
            eprintln!("check {:?} no input {:?}", it.0, it.1);
        }
        for it in self.none_at.iter().filter(|e| set.insert(e.0)) {
            eprintln!("check {:?} none at {:?}", it.0, it.1);
        }
    }
}

fn input_output_hookup_system(
    inputs: Query<(&MapPos, &SingleInput)>,
    mut outputs: Query<(Entity, &MapPos, &mut MultipleOutputs)>,
    map: Res<MapCache>,
    mut debug: Local<DebugIOHookupSystem>,
) {
    for (o_entity, pos, mut outputs) in outputs.iter_mut() {
        // NOTE to make sure not to trigger unnecessary change detection
        //      this loop uses an index range to not trigger a mut deref of Mut<> prematurely
        for i in 0..outputs.outputs.len() {
            let output = &outputs.outputs[i];
            let other_pos = (*pos + output.pos).step(output.dir);

            if output.entity.is_some() {
                continue;
            } else if let Some(input_entity) = map.at(&other_pos) {
                if let Some(input) = inputs.get_component::<SingleInput>(input_entity).ok() {
                    if input.dir == output.dir.opposite() {
                        outputs.outputs[i].entity = Some(input_entity);

                        debug.hookup.push((o_entity, input_entity));
                    } else {
                        debug.wrong_dir.push((o_entity, output.dir, input.dir));
                    }
                } else {
                    debug.no_input.push((o_entity, other_pos));
                }
            } else {
                debug.none_at.push((o_entity, other_pos));
            }
        }
    }

    debug.print();
}

fn output_item_stuff_hookup_system(
    mut entities: Query<
        (
            (Entity, &MultipleOutputs),
            (
                Option<&mut Merger>,
                Option<&mut RandomItemGenerator>,
                Option<&mut Belt>,
            ),
        ),
        Changed<MultipleOutputs>,
    >,
) {
    for ((entity, outputs), it) in entities.iter_mut() {
        if let Some(mut merger) = it.0 {
            let len = outputs.outputs.len();
            merger.outputs.resize(len, Entity::new(0));
            for i in 0..len {
                if let Some(e) = outputs.outputs[i].entity {
                    merger.outputs[i] = e;
                }
            }
            println!("output  {:?} set to {:?}", entity, merger.outputs);
        } else if let Some(mut item_gen) = it.1 {
            item_gen.output = outputs.outputs[0].entity;
            println!("output  {:?} set to {:?}", entity, item_gen.output);
        } else if let Some(mut belt) = it.2 {
            belt.output = outputs.outputs[0].entity;
            println!("output  {:?} set to {:?}", entity, belt.output);
        }
    }
}

//////

#[derive(Default)]
struct DrawItems {
    entities: Vec<Entity>,
}

fn draw_belt_system(
    belts: Query<&Belt>,
    mut shapes: Query<(&mut Transform, &mut ShapeColors)>,
    mut draw_items: Local<DrawItems>,
    mut cmds: Commands,
) {
    let mut index = 0;

    for belt in belts.iter() {
        for item in belt.items() {
            let item: &BeltItem = item;
            let (pos, dir) = belt.location_on_path(item.pos) as (Vec3, Vec3);

            if draw_items.entities.len() < index + 1 {
                let entity = cmds
                    .spawn_bundle(lyon().circle(2.0).outlined_pos3(
                        item.color(),
                        Color::BLACK,
                        1.0,
                        pos.truncate().extend(5.0),
                    ))
                    .id();
                draw_items.entities.push(entity);
            } else if let Ok((mut transform, mut colors)) = shapes.get_mut(draw_items.entities[index]) {
                transform.translation.x = pos.x;
                transform.translation.y = pos.y;
                colors.main = item.color();
            }

            index += 1;
        }
    }
}
