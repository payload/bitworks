use bevy::prelude::*;

use crate::{
    belt_advance_items_system, belt_input_system, merger_system, null_sink_system,
    random_item_generator_system, AppState,
};

pub struct BeltPlugin;
impl Plugin for BeltPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::PreUpdate, belt_input_system.system())
            .add_system_set(
                SystemSet::on_update(AppState::GameRunning)
                    .with_system(belt_advance_items_system.system())
                    .with_system(null_sink_system.system())
                    .with_system(random_item_generator_system.system())
                    .with_system(merger_system.system()),
            );
    }
}
