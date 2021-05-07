use bevy::utils::HashMap;

use crate::core::*;

#[derive(Default)]
pub struct ItemAcceptor {
    slots: HashMap<usize, ItemAcceptorSlot>,
}
pub struct ItemAcceptorSlot {
    pos: Pos,
    dir: Dir,
    filter: ItemFilter,
}

impl ItemAcceptor {
    pub fn slot(&mut self, slot: usize) -> &mut ItemAcceptorSlot {
        self.slots.insert(
            slot,
            ItemAcceptorSlot {
                pos: (0, 0),
                dir: Dir::S,
                filter: ItemFilter::Color,
            },
        );
        self.slots.get_mut(&slot).unwrap()
    }

    pub fn take_slot(&mut self, slot: usize) -> Option<Item> {
        None
    }
}

impl ItemAcceptorSlot {
    pub fn pos(&mut self, x: usize, y: usize) -> &mut Self {
        self.pos = (x, y);
        self
    }

    pub fn dir(&mut self, dir: Dir) -> &mut Self {
        self.dir = dir;
        self
    }

    fn filter(&mut self, filter: ItemFilter) -> &mut Self {
        self.filter = filter;
        self
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
}
