use crate::prelude::*;

#[system(for_each)]
#[read_component(MyTurn)]
#[read_component(WantsToApproach)]
#[write_component(Point)]
#[write_component(FieldOfView)]
#[write_component(EntityMoved)]
#[read_component(TileSize)]
#[filter(component::<MyTurn>())]
pub fn approach(
    _ecs: &SubWorld,
    entity: &Entity,
    wants_approach: &WantsToApproach,
    _fov: &mut FieldOfView,
    pos: &mut Point,
    size: Option<&TileSize>,
    #[resource] map: &mut Map,
    commands: &mut CommandBuffer,
) {
    let approach_pos = map.index_to_point2d(wants_approach.idx);
    let closest_point = if let Some(size) = size {
        let rect = Rect::with_size(pos.x, pos.y, size.x, size.y);
        let mut options: Vec<_> = rect
            .point_set()
            .iter()
            .map(|p| {
                (
                    *p,
                    DistanceAlg::PythagorasSquared.distance2d(*p, approach_pos),
                )
            })
            .collect();
        options.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        options[0].0
    } else {
        *pos
    };
    let offset = *pos - closest_point;
    let start = map.point2d_to_index(closest_point);
    let path = a_star_search(start, wants_approach.idx, map);
    if path.success && path.steps.len() > 1 {
        let new_idx = path.steps[1];
        let destination = map.index_to_point2d(new_idx) + offset;
        commands.add_component(*entity, WantsToMove { destination });
    }

    commands.remove_component::<WantsToApproach>(*entity);
    commands.remove_component::<MyTurn>(*entity);
    // map.debug_pathing = false;
}
