use bevy::prelude::*;

use crate::{Belt, BeltItem, Item, ItemInput, try_push_item_to_input};

pub struct RandomItemGenerator {
    pub next_time: f64,
    pub cooldown: f32,
    pub output: Option<Entity>,
}

pub fn random_item_generator_system(
    mut generators: Query<&mut RandomItemGenerator>,
    mut item_inputs: Query<&mut ItemInput>,
    time: Res<Time>,
) {
    let time = time.seconds_since_startup();

    for mut generator in generators.iter_mut() {
        if generator.next_time <= time {
            if let Some(output) = generator.output {
                if let Ok(mut item_input) = item_inputs.get_mut(output) {
                    let mut gen_item = BeltItem::new(0.0, Item::random());
                    if try_push_item_to_input(&mut gen_item, &mut item_input) {
                        generator.next_time = time + generator.cooldown as f64;
                    }
                } else {
                    eprintln!("failed  {:?} output item to {:?}", 1, 2);
                    generator.output = None;
                }
            }
        }
    }
}
