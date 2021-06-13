use bevy::prelude::*;

use crate::{Belt, ItemInput};

pub struct Merger {
    pub inputs: Vec<Entity>,
    pub outputs: Vec<Entity>,

    pub next_time: f64,
    pub cooldown: f32,
    pub items_per_step: usize,
    pub input_cursor: usize,
    pub output_cursor: usize,
}

pub fn merger_system(
    mut mergers: Query<&mut Merger>,
    mut inputs: Query<&mut ItemInput>,
    mut belts: Query<&mut Belt>,
    time: Res<Time>,
) {
    let time = time.seconds_since_startup();

    for mut merger in mergers.iter_mut() {
        if merger.next_time <= time {
            merger
                .inputs
                .drain_filter(|it| inputs.get_mut(*it).is_err());
            merger
                .outputs
                .drain_filter(|it| belts.get_mut(*it).is_err());

            if merger.input_cursor >= merger.inputs.len() {
                merger.input_cursor = 0;
            }
            if merger.output_cursor >= merger.outputs.len() {
                merger.output_cursor = 0;
            }

            if merger.inputs.is_empty() || merger.outputs.is_empty() {
                continue;
            }

            // CHECKED: inputs and outputs exist, vecs are non-empty, cursors in range

            let in_len = merger.inputs.len();
            let out_len = merger.outputs.len();
            let mut did_something = false;

            'item_loop: for _ in 0..merger.items_per_step {
                // try every input beginning from input cursor
                //  then try every output beginning from output cursor
                //   if item is passed on set output cursor and break output loop
                let in_cursor = merger.input_cursor;
                let out_cursor = merger.output_cursor;
                let mut passed_on = false;

                'input_loop: for index in (in_cursor..in_len).chain(0..in_cursor) {
                    let input_e = *merger.inputs.get(index).expect("checked");
                    let mut input = inputs.get_mut(input_e).expect("checked");

                    if let Some(item) = input.oldest_item().cloned() {
                        'output_loop: for index in (out_cursor..out_len).chain(0..out_cursor) {
                            let output_e = *merger.outputs.get(index).expect("checked");
                            let mut belt = belts.get_mut(output_e).expect("checked");

                            if belt.is_space(&item) {
                                if let Some(item) = input.pop_oldest_item() {
                                    belt.add_item(item);
                                    merger.output_cursor = (index + 1) % out_len;
                                    passed_on = true;
                                    break 'output_loop;
                                }
                            }
                        }

                        if passed_on {
                            merger.input_cursor = (index + 1) % in_len;
                            break 'input_loop;
                        }
                    }
                }

                if passed_on {
                    did_something = true;
                } else {
                    break 'item_loop;
                }
            }

            if did_something {
                merger.next_time = time + merger.cooldown as f64;
            }
        }
    }
}
