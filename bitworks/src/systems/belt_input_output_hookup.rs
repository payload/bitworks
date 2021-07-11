use bevy::{prelude::*, utils::HashSet};

use crate::{Belt, CompassDir, MapCache, MapPos, Merger, RandomItemGenerator};

pub struct SingleInput {
    pub pos: MapPos,
    pub dir: CompassDir,
}

pub fn input(pos: MapPos, dir: CompassDir) -> SingleInput {
    SingleInput { pos, dir }
}

pub struct SingleOutput {
    pub pos: MapPos,
    pub dir: CompassDir,
    pub entity: Option<Entity>,
}

pub struct MultipleOutputs {
    pub outputs: Vec<SingleOutput>,
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

pub fn output<P: Into<MapPos>>(pos: P, dir: CompassDir) -> MultipleOutputs {
    MultipleOutputs {
        outputs: vec![SingleOutput {
            pos: pos.into(),
            dir,
            entity: None,
        }],
    }
}

pub fn outputs(entries: &[(MapPos, CompassDir)]) -> MultipleOutputs {
    MultipleOutputs::new(entries)
}

pub struct BeltInputOutputHookupPlugin;

impl Plugin for BeltInputOutputHookupPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            input_output_hookup_system.system().label("io_hookup"),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            output_item_stuff_hookup_system.system().after("io_hookup"),
        );
    }
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
            debug!("check {:?} hookup {:?}", it.0, it.1);
        }
        for it in self.wrong_dir.iter().filter(|e| set.insert(e.0)) {
            debug!("check {:?} wrong dir {:?} to {:?}", it.0, it.1, it.2);
        }
        for it in self.no_input.iter().filter(|e| set.insert(e.0)) {
            debug!("check {:?} no input {:?}", it.0, it.1);
        }
        for it in self.none_at.iter().filter(|e| set.insert(e.0)) {
            debug!("check {:?} none at {:?}", it.0, it.1);
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
    for ((_entity, outputs), it) in entities.iter_mut() {
        if let Some(mut merger) = it.0 {
            let len = outputs.outputs.len();
            merger.outputs.resize(len, Entity::new(0));
            for i in 0..len {
                if let Some(e) = outputs.outputs[i].entity {
                    merger.outputs[i] = e;
                }
            }
            // debug!("output  {:?} set to {:?}", entity, merger.outputs);
        } else if let Some(mut item_gen) = it.1 {
            item_gen.output = outputs.outputs[0].entity;
            // debug!("output  {:?} set to {:?}", entity, item_gen.output);
        } else if let Some(mut belt) = it.2 {
            belt.output = outputs.outputs[0].entity;
            // debug!("output  {:?} set to {:?}", entity, belt.output);
        }
    }
}
