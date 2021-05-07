use bevy::{prelude::*, utils::HashMap};

use crate::core::*;

#[derive(Default)]
pub struct MapCache {
    pos_cache: HashMap<Pos, Entity>,
    entity_cache: HashMap<Entity, Pos>,
}

pub fn map_cache_system(
    mut map: ResMut<MapCache>,
    pos: Query<(Entity, &Pos, &String), Added<Pos>>,
) {
    for (e, pos, name) in pos.iter() {
        map.entity_cache.insert(e, pos.clone());
        map.pos_cache.insert(pos.clone(), e);
    }
}

pub fn map_cache_gc_system(mut map: ResMut<MapCache>, removed: RemovedComponents<Pos>) {
    for e in removed.iter() {
        map.entity_cache
            .remove(&e)
            .and_then(|pos| map.pos_cache.remove(&pos));
    }
}
