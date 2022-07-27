use crate::prelude::*;

#[system(for_each)]
#[read_component(MyTurn)]
#[write_component(Point)]
#[read_component(Chasing)]
#[write_component(FieldOfView)]
#[filter(component::<MyTurn>())]
pub fn chasing(
    ecs: &SubWorld,
    entity: &Entity,
    pos: &mut Point,
    fov: &mut FieldOfView,
    chasing: &Chasing,
    #[resource] map: &mut Map,
    commands: &mut CommandBuffer,
) {
    // Is the target still valid?
    if let Ok(target) = ecs.entry_ref(chasing.target) {
        let target_pos = target.get_component::<Point>().unwrap();
        let my_idx = map.point2d_to_index(*pos);
        let to_idx = map.point2d_to_index(*target_pos);
        let path = a_star_search(my_idx, to_idx, &*map);
        if path.success && path.steps.len() < 15 {
            let old_idx = map.point2d_to_index(*pos);
            let new_idx = path.steps[1];
            crate::spatial::move_entity(*entity, old_idx, new_idx);
            *pos = map.index_to_point2d(new_idx);
            fov.is_dirty = true;
            commands.add_component(*entity, EntityMoved);

            // All done
            commands.remove_component::<MyTurn>(*entity);
        }
    }

    // stop chasing
    commands.remove_component::<Chasing>(*entity);
    // not removing MyTurn -- fall through to default movement
}
