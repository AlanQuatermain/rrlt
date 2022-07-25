use crate::prelude::*;

#[system(for_each)]
#[read_component(MyTurn)]
#[read_component(WantsToFlee)]
#[write_component(Point)]
#[write_component(FieldOfView)]
#[write_component(EntityMoved)]
#[filter(component::<MyTurn>())]
pub fn flee(
    ecs: &SubWorld,
    entity: &Entity,
    flee: &WantsToFlee,
    pos: &mut Point,
    fov: &mut FieldOfView,
    #[resource] map: &mut Map,
    commands: &mut CommandBuffer,
) {
    let start = map.point2d_to_index(*pos);
    let flee_map = DijkstraMap::new(map.width, map.height, &flee.indices, &*map, 12.0);
    let flee_target = DijkstraMap::find_highest_exit(&flee_map, start, &*map);
    if let Some(flee_target) = flee_target {
        if !map.blocked[flee_target] {
            map.blocked[start] = false;
            map.blocked[flee_target] = true;
            fov.is_dirty = true;
            *pos = map.index_to_point2d(flee_target);
            commands.add_component(*entity, EntityMoved);
        }
    }

    commands.remove_component::<WantsToFlee>(*entity);
    commands.remove_component::<MyTurn>(*entity);
}
