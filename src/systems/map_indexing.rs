use crate::prelude::*;

#[system]
#[read_component(BlocksTile)]
#[read_component(Point)]
#[read_component(WantsToMove)]
pub fn map_indexing(ecs: &SubWorld, #[resource] map: &mut Map) {
    let movers: Vec<Entity> = <&WantsToMove>::query()
        .iter(ecs)
        .map(|m| m.entity)
        .collect();
    let mut blockers = <(&Point, Entity)>::query().filter(component::<BlocksTile>());
    map.populate_blocked();
    blockers.for_each(ecs, |(pos, entity)| {
        // println!("{:?} is blocking {:?}", entity, pos);
        if !movers.contains(entity) {
            let idx = map.point2d_to_index(*pos);
            map.blocked[idx] = true;
        }
    });
}
