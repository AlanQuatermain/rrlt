use crate::prelude::*;

#[system(for_each)]
#[read_component(MyTurn)]
#[write_component(MoveMode)]
#[write_component(Point)]
#[write_component(FieldOfView)]
#[write_component(EntityMoved)]
#[filter(component::<MyTurn>()&!component::<Player>())]
pub fn default_movement(
    _ecs: &SubWorld,
    entity: &Entity,
    pos: &mut Point,
    _fov: &mut FieldOfView,
    mode: &mut MoveMode,
    #[resource] map: &mut Map,
    #[resource] rng: &mut RandomNumberGenerator,
    commands: &mut CommandBuffer,
) {
    commands.remove_component::<MyTurn>(*entity);
    match &mut mode.0 {
        Movement::Static | Movement::Immobile => {}
        Movement::Random => {
            let delta = match rng.roll_dice(1, 5) {
                1 => Point::new(-1, 0),
                2 => Point::new(1, 0),
                3 => Point::new(0, -1),
                4 => Point::new(0, 1),
                _ => Point::zero(),
            };
            let new_pos = *pos + delta;
            if map.can_enter_tile(new_pos) {
                commands.add_component(
                    *entity,
                    WantsToMove {
                        destination: new_pos,
                    },
                );
            }
        }
        Movement::RandomWaypoint { path } => {
            let idx = map.point2d_to_index(*pos);
            if let Some(path) = path {
                if path.len() > 1 {
                    let new_idx = path[1];
                    let destination = map.index_to_point2d(new_idx);
                    commands.add_component(*entity, WantsToMove { destination });
                    path.remove(0);
                } else {
                    mode.0 = Movement::RandomWaypoint { path: None }
                }
            } else {
                let target_x = rng.range(1, map.width as i32 - 2);
                let target_y = rng.range(1, map.height as i32 - 2);
                let target = map.point2d_to_index(Point::new(target_x, target_y));
                if map.tiles[target].is_walkable() {
                    let path = a_star_search(idx, target, &mut *map);
                    if path.success && path.steps.len() > 1 {
                        mode.0 = Movement::RandomWaypoint {
                            path: Some(path.steps),
                        }
                    }
                }
            }
        }
    }
}
