use crate::game_types::*;
use bevy::prelude::*;

pub struct RandomItemGenerator {
    pub next_time: f64,
    pub cooldown: f32,
    pub output: Option<Entity>,
}

pub fn random_item_generator_system(
    mut generators: Query<&mut RandomItemGenerator>,
    mut belts: Query<&mut Belt>,
    time: Res<Time>,
) {
    let time = time.seconds_since_startup();

    for mut generator in generators.iter_mut() {
        if generator.next_time <= time {
            if let Some(output) = generator.output {
                if let Ok(mut belt) = belts.get_mut(output) {
                    let gen_item = BeltItem::new(0.0, Item::random());

                    if belt.is_space(&gen_item) {
                        belt.add_item(gen_item);
                        generator.next_time = time + generator.cooldown as f64;
                    }
                }
            }
        }
    }
}
