use bevy::utils::HashMap;

use crate::core::*;

#[derive(Default)]
pub struct ItemEjector {
    slots: HashMap<usize, ItemEjectorSlot>,
}

#[derive(Default)]
pub struct ItemEjectorSlot {
    pub items: Vec<Item>,
    pub max_items: usize,
    pub pos: Pos,
    pub dir: Dir,
}

impl ItemEjector {
    pub fn slot(&mut self, slot: usize) -> &mut ItemEjectorSlot {
        self.slots.insert(slot, ItemEjectorSlot::default());
        self.slots.get_mut(&slot).unwrap()
    }

    pub fn slots(&self) -> impl Iterator<Item = (&usize, &ItemEjectorSlot)> {
        self.slots.iter()
    }

    pub fn eject(&mut self, slot: usize, item: Item) {}
}

impl ItemEjectorSlot {
    pub fn pos(&mut self, x: usize, y: usize) -> &mut Self {
        self.pos = Pos(x, y);
        self
    }

    pub fn dir(&mut self, dir: Dir) -> &mut Self {
        self.dir = dir;
        self
    }

    pub fn get_pos(&self) -> &Pos {
        &self.pos
    }

    pub fn get_dir(&self) -> Dir {
        self.dir
    }

    pub fn is_free(&self) -> bool {
        self.items.len() < self.max_items
    }

    pub fn has_item(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn item(&self) -> Option<&Item> {
        self.items.get(0)
    }
}
