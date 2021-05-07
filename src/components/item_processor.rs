use bevy::prelude::*;

use crate::buildings::*;
use crate::components::*;

impl ItemProcessor {
    pub fn process(
        &mut self,
        me: Entity,
        acceptor: Mut<ItemAcceptor>,
        ejector: Mut<ItemEjector>,
        wiring: Mut<Wiring>,
    ) -> Option<()> {
        match self.label {
            ProcessorLabel::Cutter4 => cutter4_process(me, self, acceptor, ejector, wiring),
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

#[allow(dead_code)]
pub enum ProcessorLabel {
    None,
    Cutter4,
    Mixer,
}

impl Default for ProcessorLabel {
    fn default() -> Self {
        Self::None
    }
}
