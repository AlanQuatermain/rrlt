use crate::prelude::*;

#[system]
#[read_component(FieldOfView)]
#[read_component(Point)]
#[read_component(Carnivore)]
#[read_component(Herbivore)]
#[read_component(Item)]
pub fn animal_ai(
    ecs: &SubWorld,
    #[resource] map: &mut Map,
    #[resource] gamelog: &mut Gamelog,
    commands: &mut CommandBuffer,
) {
    map.populate_blocked();

    // Herbivores run away a lot
    <(Entity, &FieldOfView, &Point)>::query()
        .filter(component::<Herbivore>())
        .for_each(ecs, |(entity, fov, pos)| {
            let run_from: Vec<Point> = <&Point>::query()
                .filter(!component::<Item>())
                .iter(ecs)
                .filter_map(|p_ref| {
                    if fov.visible_tiles.contains(p_ref) {
                        Some(*p_ref)
                    } else {
                        None
                    }
                })
                .collect();

            if run_from.is_empty() {
                return;
            }
            let my_idx = map.point2d_to_index(*pos);
            let starts: Vec<usize> = run_from.iter().map(|p| map.point2d_to_index(*p)).collect();

            let flee_map = DijkstraMap::new(map.width, map.height, &starts, &*map, 20.0);
            if let Some(flee_target) = DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map) {
                let destination = map.index_to_point2d(flee_target);
                commands.push((
                    (),
                    WantsToMove {
                        entity: *entity,
                        destination,
                    },
                ));
            }
        });

    // Carnivores just want to eat everything
    <(Entity, &FieldOfView, &Point)>::query()
        .filter(component::<Carnivore>())
        .for_each(ecs, |(entity, fov, pos)| {
            let mut attacked = false;
            let run_toward: Vec<Point> = <(&Point, Entity)>::query()
                .filter(component::<Herbivore>() | component::<Player>())
                .iter(ecs)
                .filter_map(|(p_ref, victim)| {
                    if fov.visible_tiles.contains(p_ref) && !attacked {
                        let distance = DistanceAlg::Pythagoras.distance2d(*pos, *p_ref);
                        if distance < 1.5 {
                            attacked = true;
                            commands.push((
                                (),
                                WantsToAttack {
                                    attacker: *entity,
                                    victim: *victim,
                                },
                            ));
                            None
                        } else {
                            Some(*p_ref)
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if run_toward.is_empty() || attacked {
                return;
            }

            let my_idx = map.point2d_to_index(*pos);
            let starts: Vec<usize> = run_toward
                .iter()
                .map(|p| map.point2d_to_index(*p))
                .collect();
            let chase_map = DijkstraMap::new(map.width, map.height, &starts, &*map, 20.0);
            if let Some(target) = DijkstraMap::find_lowest_exit(&chase_map, my_idx, &*map) {
                let destination = map.index_to_point2d(target);
                commands.push((
                    (),
                    WantsToMove {
                        entity: *entity,
                        destination,
                    },
                ));
            }
        });
}
