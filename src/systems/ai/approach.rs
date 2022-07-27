use crate::prelude::*;

#[system(for_each)]
#[read_component(MyTurn)]
#[read_component(WantsToApproach)]
#[write_component(Point)]
#[write_component(FieldOfView)]
#[write_component(EntityMoved)]
#[filter(component::<MyTurn>())]
pub fn approach(
    ecs: &SubWorld,
    entity: &Entity,
    wants_approach: &WantsToApproach,
    fov: &mut FieldOfView,
    pos: &mut Point,
    #[resource] map: &mut Map,
    commands: &mut CommandBuffer,
) {
    let start = map.point2d_to_index(*pos);
    let path = a_star_search(start, wants_approach.idx, map);
    if path.success && path.steps.len() > 1 {
        let old_idx = map.point2d_to_index(*pos);
        let new_idx = path.steps[1];
        if !crate::spatial::is_blocked(new_idx) {
            crate::spatial::move_entity(*entity, old_idx, new_idx);
            *pos = map.index_to_point2d(new_idx);
            fov.is_dirty = true;
            commands.add_component(*entity, EntityMoved);
        }
    }

    commands.remove_component::<WantsToApproach>(*entity);
    commands.remove_component::<MyTurn>(*entity);
    // map.debug_pathing = false;
}
