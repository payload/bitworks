use bevy::utils::HashMap;

use crate::core::*;

#[derive(Default)]
pub struct ItemEjector {
    slots: HashMap<usize, ItemEjectorSlot>,
}

#[derive(Default)]
pub struct ItemEjectorSlot {
    items: Vec<Item>,
    max_items: usize,
    pos: Pos,
    dir: Dir,
}

impl ItemEjector {
    pub fn slot(&mut self, slot: usize) -> &mut ItemEjectorSlot {
        self.slots.insert(slot, ItemEjectorSlot::default());
        self.slots.get_mut(&slot).unwrap()
    }

    pub fn slots(&self) -> impl Iterator<Item = &ItemEjectorSlot> {
        [].iter()
    }

    pub fn eject(&mut self, slot: usize, item: Item) {}
}

impl ItemEjectorSlot {
    pub fn pos(&mut self, x: usize, y: usize) -> &mut Self {
        self.pos = (x, y);
        self
    }

    pub fn dir(&mut self, dir: Dir) -> &mut Self {
        self.dir = dir;
        self
    }

    pub fn is_free(&self) -> bool {
        self.items.len() < self.max_items
    }
}
