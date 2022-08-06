use crate::prelude::*;

#[system]
#[read_component(Point)]
#[write_component(FieldOfView)]
#[read_component(Player)]
#[read_component(BlocksVisibility)]
#[read_component(TileSize)]
pub fn fov(ecs: &mut SubWorld, #[resource] map: &mut Map) {
    let mut updated_locations: Vec<Point> = Vec::new();

    map.view_blocked.clear();
    <&Point>::query()
        .filter(component::<BlocksVisibility>())
        .for_each(ecs, |pos| {
            let idx = map.point2d_to_index(*pos);
            map.view_blocked.insert(idx);
        });

    let mut views = <(&Point, Option<&TileSize>, &mut FieldOfView, Entity)>::query();
    views
        .iter_mut(ecs)
        .filter(|(_, _, fov, _)| fov.is_dirty)
        .for_each(|(pos, size, mut fov, _)| {
            if let Some(size) = size {
                fov.visible_tiles.clear();
                let rect = Rect::with_size(pos.x, pos.y, size.x, size.y);
                for loc in rect.point_set().iter() {
                    fov.visible_tiles
                        .extend(&field_of_view_set(*loc, fov.radius, map));
                }
            } else {
                fov.visible_tiles = field_of_view_set(*pos, fov.radius, map);
            }
            fov.is_dirty = false;
            updated_locations.push(*pos);
        });

    let mut is_player = false;
    <(&Point, &FieldOfView)>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .filter(|(pos, _)| updated_locations.contains(*pos))
        .for_each(|(_, fov)| {
            is_player = true;
            fov.visible_tiles.iter().for_each(|pos| {
                let idx = map.point2d_to_index(*pos);
                map.revealed_tiles[idx] = true;
            });
        });
}
