use bevy::prelude::*;

use crate::buildings::*;
use crate::components::*;

#[allow(dead_code)]
pub enum ProcessorLabel {
    None,
    Paintcutter,
    Mixer,
}

#[derive(Default)]
pub struct ItemProcessor {
    pub label: ProcessorLabel,
}

impl ItemProcessor {
    pub fn label(&mut self, label: ProcessorLabel) -> &mut Self {
        self.label = label;
        self
    }
}

impl Default for ProcessorLabel {
    fn default() -> Self {
        Self::None
    }
}
