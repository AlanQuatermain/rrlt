use crate::prelude::*;

#[system]
#[read_component(BlocksTile)]
#[read_component(Point)]
pub fn map_indexing(ecs: &SubWorld, #[resource] map: &mut Map) {
    let mut blockers = <(&Point, Entity)>::query().filter(component::<BlocksTile>());
    map.populate_blocked();
    blockers.for_each(ecs, |(pos, entity)| {
        // println!("{:?} is blocking {:?}", entity, pos);
        let idx = map.point2d_to_index(*pos);
        map.blocked[idx] = true;
    });
}
