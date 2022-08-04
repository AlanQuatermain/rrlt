use crate::{prelude::*, spatial};

#[system]
#[read_component(BlocksTile)]
#[read_component(Point)]
#[read_component(Pools)]
#[read_component(TileSize)]
pub fn map_indexing(ecs: &SubWorld, #[resource] map: &mut Map) {
    spatial::clear();
    spatial::populate_blocked_from_map(map);

    <(
        &Point,
        Entity,
        Option<&BlocksTile>,
        Option<&Pools>,
        Option<&TileSize>,
    )>::query()
    .for_each(ecs, |(pos, entity, blocks, stats, maybe_size)| {
        // don't index dead things
        if let Some(stats) = stats {
            if stats.hit_points.current < 1 {
                return;
            }
        }
        let size = maybe_size.map(|s| *s).unwrap_or_default();
        let idx = map.point2d_to_index(*pos);
        spatial::index_entity(*entity, idx, blocks.is_some(), size.x, size.y, map.width);
    });
}
