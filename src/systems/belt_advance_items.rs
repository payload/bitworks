use std::slice::{Iter, IterMut};

use bevy::{math::vec3, prelude::*};

///////////////////////////////////////////////////////////////////////////////

struct BeltInput {
    space: f32,
    items: Vec<BeltItem>,
    capacity: usize,
}

impl BeltInput {
    fn space(&self) -> usize {
        self.capacity.saturating_sub(self.items.len())
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct ItemInput {
    items: Vec<BeltItem>,
    capacity: usize,
}

impl ItemInput {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: Vec::new(),
        }
    }

    pub fn space(&self) -> usize {
        self.capacity.saturating_sub(self.items.len())
    }

    pub fn clear_items(&mut self) {
        self.items.clear();
    }

    pub fn add_items<T: IntoIterator<Item = BeltItem>>(&mut self, items: T) {
        self.items.extend(items)
    }

    pub fn oldest_item(&self) -> Option<&BeltItem> {
        self.items.last()
    }

    pub fn pop_oldest_item(&mut self) -> Option<BeltItem> {
        self.items.pop()
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct BeltItem {
    pub pos: f32,
    pub item: Item,
}

impl BeltItem {
    pub fn new(pos: f32, item: Item) -> Self {
        Self { pos, item }
    }

    pub fn red(pos: f32) -> Self {
        Self::new(pos, Item::Red)
    }

    pub fn green(pos: f32) -> Self {
        Self::new(pos, Item::Green)
    }

    pub fn padding(&self) -> f32 {
        1.0
    }
}

impl std::ops::Deref for BeltItem {
    type Target = Item;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct BeltSegment {
    pub start: Vec3,
    pub end: Vec3,
}

impl BeltSegment {
    pub fn straight(startx: i32, starty: i32, endx: i32, endy: i32) -> Self {
        Self {
            start: vec3(startx as f32, starty as f32, 0.0),
            end: vec3(endx as f32, endy as f32, 0.0),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum Item {
    Red,
    Green,
}

impl Item {
    pub fn color(&self) -> Color {
        match &self {
            Item::Red => Color::RED,
            Item::Green => Color::GREEN,
        }
    }

    pub fn random() -> Self {
        use Item::*;
        let items = [Red, Green];
        items[fastrand::usize(0..items.len())]
    }
}

///////////////////////////////////////////////////////////////////////////////

pub struct Belt {
    pub segments: Vec<BeltSegment>,
    pub items: Vec<BeltItem>,
    pub output: Option<Entity>,
}

impl Belt {
    pub fn items(&self) -> Iter<BeltItem> {
        self.items.iter()
    }

    pub fn items_mut(&mut self) -> IterMut<BeltItem> {
        self.items.iter_mut()
    }

    pub fn segments(&self) -> impl Iterator<Item = &BeltSegment> {
        self.segments.iter()
    }

    pub fn output(&self) -> Option<Entity> {
        self.output
    }

    pub fn add_item(&mut self, item: BeltItem) {
        let index = self
            .items
            .binary_search_by(|other| other.pos.total_cmp(&item.pos))
            .map_or_else(|i| i, |i| i);
        self.items.insert(index, item);
    }

    pub fn pass_on(&mut self, count: usize) -> Vec<BeltItem> {
        self.items.split_off(self.items.len() - count)
    }

    /// go through each segment, accumulate segment lengths,
    /// until there is the segment with this pos
    /// and return Vec3 pos with direction
    /// or else return end or zero
    pub fn location_on_path(&self, pos: f32) -> (Vec3, Vec3) {
        let mut accu = 0.0;

        for segment in self.segments.iter() {
            let diff = segment.end - segment.start;
            let length = diff.length();
            let dir = diff.normalize_or_zero();
            let segment_pos = pos - accu;

            if segment_pos >= 0.0 && segment_pos <= length {
                return (segment.start + dir * segment_pos, dir);
            } else {
                accu += length;
            }
        }

        if let Some(segment) = self.segments.last() {
            let diff = segment.end - segment.start;
            let dir = diff.normalize_or_zero();
            (segment.end, dir)
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        }
    }

    pub fn total_length(&self) -> f32 {
        self.segments
            .iter()
            .fold(0.0, |acc, seg| acc + seg.start.distance(seg.end))
    }

    pub fn is_space(&self, item: &BeltItem) -> bool {
        if let Some(first) = self.items.first() {
            item.padding() <= first.pos - first.padding()
        } else {
            true
        }
    }
}

pub fn belt_advance_items_system(
    mut belts: Query<&mut Belt>,
    mut item_inputs: Query<&mut ItemInput>,
    time: Res<Time>,
) {
    let time = time.delta_seconds();

    for mut belt in belts.iter_mut() {
        let speed = 10.0;
        let advance = speed * time;

        let total_length = belt.total_length();
        let mut next_stop = if belt.output.is_some() {
            NextStop::Output
        } else {
            NextStop::End
        };

        let mut item_input = belt.output.and_then(|e| item_inputs.get_mut(e).ok());

        for i in (0..belt.items.len()).rev() {
            belt.items[i].pos += advance;

            match next_stop {
                NextStop::End => {
                    let item = &mut belt.items[i];
                    item.pos = total_length.min(item.pos);
                    next_stop = NextStop::Item(item.pos - item.padding());
                }
                NextStop::Item(stop) => {
                    let item = &mut belt.items[i];
                    item.pos = (stop - item.padding()).min(item.pos);
                    next_stop = NextStop::Item(item.pos - item.padding());
                }
                NextStop::Output => {
                    if belt.items[i].pos > total_length
                        && push_item_to_input(&mut *belt, item_input.as_mut().unwrap())
                    {
                        // popped one
                    } else {
                        next_stop = NextStop::Item(belt.items[i].pos - belt.items[i].padding());
                    }
                }
            }
        }
    }
}

fn push_item_to_input(belt: &mut Belt, input: &mut ItemInput) -> bool {
    if !belt.items.is_empty() {
        if input.space() > 0 {
            let mut item = belt.items.pop().unwrap();
            item.pos -= belt.total_length();
            input.items.insert(0, item);
            true
        } else {
            belt.items.last_mut().unwrap().pos = belt.total_length();
            false
        }
    } else {
        false
    }
}

enum NextStop {
    End,
    Item(f32),
    Output,
}
