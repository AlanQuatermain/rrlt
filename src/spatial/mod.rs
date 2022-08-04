use crate::prelude::*;
use std::sync::Mutex;

#[derive(Default)]
struct SpatialMap {
    blocked: Vec<(bool, bool)>,
    tile_content: Vec<Vec<(Entity, bool)>>,
}

impl SpatialMap {
    fn new() -> Self {
        Self {
            blocked: Vec::new(),
            tile_content: Vec::new(),
        }
    }
}

lazy_static! {
    static ref SPATIAL_MAP: Mutex<SpatialMap> = Mutex::new(SpatialMap::new());
}

pub fn set_size(map_tile_count: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked = vec![(false, false); map_tile_count];
    lock.tile_content = vec![Vec::new(); map_tile_count];
}

#[allow(dead_code)]
pub fn set_blocked(idx: usize, by_entity: bool) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked[idx] = (true, by_entity);
}

pub fn index_entity(
    entity: Entity,
    idx: usize,
    blocks_tile: bool,
    size_x: i32,
    size_y: i32,
    map_width: usize,
) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    for x in 0..size_x as usize {
        for y in 0..size_y as usize {
            let off_idx = idx + x + (y * map_width);
            lock.tile_content[off_idx].push((entity, blocks_tile));
            if blocks_tile {
                lock.blocked[off_idx].1 = true;
            }
        }
    }
}

pub fn clear() {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked.iter_mut().for_each(|b| {
        b.0 = false;
        b.1 = false
    });
    for content in lock.tile_content.iter_mut() {
        content.clear();
    }
}

pub fn populate_blocked_from_map(map: &Map) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    for (i, tile) in map.tiles.iter().enumerate() {
        lock.blocked[i].0 = !tile.is_walkable();
    }
}

pub fn is_blocked(idx: usize) -> bool {
    let lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked[idx].0 || lock.blocked[idx].1
}

#[allow(dead_code)]
pub fn is_blocked_by_entity(idx: usize) -> bool {
    SPATIAL_MAP.lock().unwrap().blocked[idx].1
}

pub fn is_blocked_by_tile(idx: usize) -> bool {
    SPATIAL_MAP.lock().unwrap().blocked[idx].0
}

#[allow(dead_code)]
pub fn is_blocked_ignoring_entity(idx: usize, entity: Entity) -> bool {
    if is_blocked_by_tile(idx) {
        return true;
    }

    if is_blocked_by_entity(idx) {
        // is there anything other than the current entity blocking in here?
        SPATIAL_MAP.lock().unwrap().tile_content[idx]
            .iter()
            .filter(|(ent, blocks)| *ent != entity && *blocks)
            .count()
            != 0
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn tiles_blocked_ignoring_entity(tiles: Vec<usize>, entity: Entity) -> bool {
    let lock = SPATIAL_MAP.lock().unwrap();
    for idx in tiles.iter() {
        if lock.blocked[*idx].0 {
            return true;
        }
    }

    for idx in tiles.iter() {
        if lock.blocked[*idx].1 {
            // is there anything other than the current entity blocking in here?
            if lock.tile_content[*idx]
                .iter()
                .filter(|(ent, blocks)| *ent != entity && *blocks)
                .count()
                != 0
            {
                return true;
            }
        }
    }

    false
}

#[allow(dead_code)]
pub fn for_each_tile_content<F>(idx: usize, mut f: F)
where
    F: FnMut(Entity),
{
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        f(entity.0);
    }
}

#[allow(dead_code)]
pub fn for_each_tile_content_until_result<T, F>(idx: usize, mut f: F) -> Option<T>
where
    F: FnMut(Entity) -> Option<T>,
{
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        if let Some(result) = f(entity.0) {
            return Some(result);
        }
    }
    None
}

pub fn move_entity(entity: Entity, moving_from: usize, moving_to: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    let mut entity_blocks = false;
    lock.tile_content[moving_from].retain(|(e, blocks)| {
        if *e == entity {
            entity_blocks = *blocks;
            false
        } else {
            true
        }
    });
    lock.tile_content[moving_to].push((entity, entity_blocks));

    // Recalculate blocks for both tiles.
    let mut from_blocked = false;
    let mut to_blocked = false;
    lock.tile_content[moving_from]
        .iter()
        .for_each(|(_, blocks)| {
            if *blocks {
                from_blocked = true;
            }
        });
    lock.tile_content[moving_to].iter().for_each(|(_, blocks)| {
        if *blocks {
            to_blocked = true;
        }
    });
    lock.blocked[moving_from].1 = from_blocked;
    lock.blocked[moving_to].1 = to_blocked;
}

pub fn remove_entity(entity: Entity, idx: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].retain(|(e, _)| *e != entity);
    let mut from_blocked = false;
    lock.tile_content[idx].iter().for_each(|(_, blocks)| {
        if *blocks {
            from_blocked = true;
        }
    });
    lock.blocked[idx].1 = from_blocked;
}
