use bevy::utils::HashMap;

use crate::core::*;

#[derive(Default)]
pub struct ItemAcceptor {
    slots: HashMap<usize, ItemAcceptorSlot>,
}
pub struct ItemAcceptorSlot {
    pub pos: Pos,
    pub dir: Dir,
    pub filter: ItemFilter,
    pub items: Vec<Item>,
    pub max_items: usize,
}

impl ItemAcceptor {
    pub fn slot(&mut self, slot: usize) -> &mut ItemAcceptorSlot {
        self.slots.insert(slot, ItemAcceptorSlot::default());
        self.slots.get_mut(&slot).unwrap()
    }

    pub fn take_slot(&mut self, slot: usize) -> Option<Item> {
        None
    }

    pub fn get_slot(&self, slot: usize) -> Option<ItemAcceptorSlot> {
        self.get_slot(slot)
    }

    pub fn slots(&self) -> impl Iterator<Item = (&usize, &ItemAcceptorSlot)> {
        self.slots.iter()
    }

    pub fn slots_mut(&mut self) -> impl Iterator<Item = (&usize, &mut ItemAcceptorSlot)> {
        self.slots.iter_mut()
    }
}

impl ItemAcceptorSlot {
    pub fn pos(&mut self, x: usize, y: usize) -> &mut Self {
        self.pos = Pos(x, y);
        self
    }

    pub fn dir(&mut self, dir: Dir) -> &mut Self {
        self.dir = dir;
        self
    }

    pub fn filter(&mut self, filter: ItemFilter) -> &mut Self {
        self.filter = filter;
        self
    }

    pub fn get_pos(&self) -> &Pos {
        &self.pos
    }

    pub fn has_color(&self) -> bool {
        false
    }

    pub fn has_shape(&self) -> bool {
        false
    }

    pub fn take(&mut self) -> Option<Item> {
        Some(Item::Red)
    }

    pub fn is_free(&self) -> bool {
        self.items.len() < self.max_items
    }
}

impl Default for ItemAcceptorSlot {
    fn default() -> Self {
        Self {
            max_items: 1,
            dir: Dir::S,
            filter: ItemFilter::All,
            items: Vec::new(),
            pos: Pos(0, 0),
        }
    }
}
