use bevy::prelude::*;

use crate::buildings::*;
use crate::components::*;

#[allow(dead_code)]
pub enum ProcessorLabel {
    None,
    Paintcutter,
    Mixer,
}

impl ItemProcessor {
    pub fn process(
        &mut self,
        me: Entity,
        acceptor: Mut<ItemAcceptor>,
        ejector: Mut<ItemEjector>,
        wiring: Mut<Wiring>,
    ) -> Option<()> {
        match self.label {
            ProcessorLabel::Paintcutter => paintcutter_process(me, self, acceptor, ejector, wiring),
            ProcessorLabel::Mixer => None,
            ProcessorLabel::None => None,
        }
    }
}

#[derive(Default)]
pub struct ItemProcessor {
    label: ProcessorLabel,
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
