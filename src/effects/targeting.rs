use crate::prelude::*;

pub fn entity_position(ecs: &SubWorld, target: Entity, map: &Map) -> Option<usize> {
    if let Ok(pos) = ecs.entry_ref(target).unwrap().get_component::<Point>() {
        Some(map.point2d_to_index(*pos))
    } else {
        None
    }
}

pub fn aoe_tiles(map: &Map, target: Point, radius: i32) -> Vec<usize> {
    let mut blast_tiles = field_of_view(target, radius, &*map);
    blast_tiles.retain(|p| map.in_bounds(*p));
    blast_tiles
        .iter()
        .map(|p| map.point2d_to_index(*p))
        .collect()
}

pub fn find_item_position(ecs: &SubWorld, target: Entity, map: &Map) -> Option<usize> {
    if let Ok(entry) = ecs.entry_ref(target) {
        // Easy - it has a position
        if let Ok(pos) = entry.get_component::<Point>() {
            return Some(map.point2d_to_index(*pos));
        }

        // Maybe it's carried?
        if let Ok(carried) = entry.get_component::<Carried>() {
            if let Ok(other_entry) = ecs.entry_ref(carried.0) {
                if let Ok(pos) = other_entry.get_component::<Point>() {
                    return Some(map.point2d_to_index(*pos));
                }
            }
        }

        // Maybe it's equipped?
        if let Ok(equipped) = entry.get_component::<Equipped>() {
            if let Ok(other_entry) = ecs.entry_ref(equipped.owner) {
                if let Ok(pos) = other_entry.get_component::<Point>() {
                    return Some(map.point2d_to_index(*pos));
                }
            }
        }
    }

    // No idea - give up
    None
}
