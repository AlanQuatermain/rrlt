use crate::prelude::*;

#[system]
#[read_component(BlocksTile)]
#[read_component(Point)]
pub fn map_indexing(
    ecs: &SubWorld,
    #[resource] map: &mut Map
) {
    let mut blockers = <(&BlocksTile, &Point)>::query();
    map.populate_blocked();
    blockers.for_each(ecs, |(_, pos)| {
        let idx = map.point2d_to_index(*pos);
        map.blocked[idx] = true;
    });
}