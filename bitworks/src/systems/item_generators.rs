use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

use crate::{try_push_item_to_input, BeltItem, Item, ItemInput};

#[derive(Default)]
pub struct RandomItemGenerator {
    pub next_time: f64,
    pub cooldown: f32,
    pub output: Option<Entity>,
}

impl Inspectable for RandomItemGenerator {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _options: Self::Attributes,
        context: &bevy_inspector_egui::Context,
    ) -> bool {
        use bevy_inspector_egui::egui;

        let mut changed = false;
        ui.vertical_centered(|ui| {
            let grid = egui::Grid::new(context.id());
            grid.show(ui, |ui| {
                ui.label("next_time");
                changed |= self
                    .next_time
                    .ui(ui, Default::default(), &context.with_id(0));
                ui.end_row();

                ui.label("cooldown");
                changed |= self
                    .cooldown
                    .ui(ui, Default::default(), &context.with_id(1));
                ui.end_row();

                ui.label("output");
                changed |= self.output.ui(ui, Default::default(), &context.with_id(2));
                ui.end_row();
            });
        });
        changed
    }

    fn setup(app: &mut AppBuilder) {
        std::primitive::f64::setup(app);
        std::primitive::f32::setup(app);
        Option::<Entity>::setup(app);
    }
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
