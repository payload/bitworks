use bevy::prelude::*;

use crate::ItemInput;

pub struct NullSink {
    inputs: Vec<Entity>,
}

impl NullSink {
    pub fn new(inputs: &[Entity]) -> Self {
        Self {
            inputs: inputs.into(),
        }
    }
}

pub fn null_sink_system(mut sinks: Query<&mut NullSink>, mut inputs: Query<&mut ItemInput>) {
    for mut sink in sinks.iter_mut() {
        sink.inputs.drain_filter(|entity| {
            if let Ok(mut input) = inputs.get_mut(*entity) {
                input.clear_items();
                false
            } else {
                true
            }
        });
    }
}
