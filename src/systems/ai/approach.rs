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
    // map.debug_pathing = true;
    println!("Pathing from {} to {}", start, wants_approach.idx);
    let path = a_star_search(start, wants_approach.idx, map);
    if path.success && path.steps.len() > 1 {
        println!("Path: {:?}", path.steps);
        map.blocked[start] = false;
        let idx = path.steps[1];
        *pos = map.index_to_point2d(path.steps[1]);
        map.blocked[idx] = true;
        fov.is_dirty = true;
        commands.add_component(*entity, EntityMoved);
    } else {
        println!(
            "Unable to find path from {:?} ({}) to {:?} ({})",
            map.index_to_point2d(start),
            start,
            map.index_to_point2d(wants_approach.idx),
            wants_approach.idx
        );
    }

    commands.remove_component::<WantsToApproach>(*entity);
    commands.remove_component::<MyTurn>(*entity);
    // map.debug_pathing = false;
}
