use bevy::{math::vec2, prelude::*, utils::HashMap};
use bevy_inspector_egui::Inspectable;

use crate::{CompassDir, SingleInput};

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Hash, Inspectable)]
pub struct MapPos {
    pub x: i32,
    pub y: i32,
}

pub struct MapPlugin;
impl Plugin for MapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(MapCache::default())
            .add_system_to_stage(CoreStage::First, map_pos_apply_transform_system.system())
            .add_system_to_stage(CoreStage::First, map_cache_system.system());
    }
}

impl MapPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn apply(&self, factor: f32, transform: &mut Transform) {
        transform.translation.x = self.x as f32 * factor;
        transform.translation.y = self.y as f32 * factor;
    }

    pub fn vec2(&self) -> Vec2 {
        vec2(self.x as f32, self.y as f32)
    }

    pub fn add_xy(&self, x: i32, y: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
        }
    }

    pub fn step(&self, dir: CompassDir) -> Self {
        match dir {
            CompassDir::N => self.add_xy(0, 1),
            CompassDir::E => self.add_xy(1, 0),
            CompassDir::S => self.add_xy(0, -1),
            CompassDir::W => self.add_xy(-1, 0),
        }
    }
}

impl From<(i32, i32)> for MapPos {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for MapPos {
    type Output = MapPos;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_xy(rhs.x, rhs.y)
    }
}

/// (x: i32, y: i32) -> MapPos
pub fn map_pos<T: Into<i32>>(x: T, y: T) -> MapPos {
    MapPos::new(x.into(), y.into())
}

#[derive(Default)]
pub struct MapCache {
    pos_cache: HashMap<MapPos, Entity>,
    entity_cache: HashMap<Entity, MapPos>,
}

impl MapCache {
    pub fn at(&self, pos: &MapPos) -> Option<Entity> {
        self.pos_cache.get(pos).map(|x| *x)
    }
}

pub fn map_cache_system(
    mut map: ResMut<MapCache>,
    pos: Query<(Entity, &MapPos), (With<SingleInput>, Added<MapPos>)>,
) {
    for (e, pos) in pos.iter() {
        map.entity_cache.insert(e, (*pos).clone());
        map.pos_cache.insert(pos.clone(), e);
    }
}

pub fn _map_cache_gc_system(mut map: ResMut<MapCache>, removed: RemovedComponents<MapPos>) {
    for e in removed.iter() {
        map.entity_cache
            .remove(&e)
            .and_then(|pos| map.pos_cache.remove(&pos));
    }
}

pub fn map_pos_apply_transform_system(
    mut query: Query<(&MapPos, &mut Transform), Changed<MapPos>>,
) {
    for (pos, mut transform) in query.iter_mut() {
        pos.apply(48.0, &mut transform);
    }
}
