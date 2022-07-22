use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(ChasingPlayer)]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn chasing(#[resource] map: &Map, ecs: &SubWorld, commands: &mut CommandBuffer) {
    let mut movers = <(Entity, &Point, &ChasingPlayer, &FieldOfView)>::query();
    let mut positions = <(Entity, &Point)>::query();
    let mut player = <(&Point, &Player)>::query();

    if movers.iter(ecs).count() == 0 {
        return;
    }

    let player_pos = player.iter(ecs).nth(0).unwrap().0;
    let player_idx = map.point2d_to_index(*player_pos);

    let search_targets = vec![player_idx];
    let mut dijkstra_map: Option<DijkstraMap> = None;

    movers.iter(ecs).for_each(|(entity, pos, _, fov)| {
        if !fov.visible_tiles.contains(&player_pos) {
            return;
        }

        let idx = map.point2d_to_index(*pos);
        let dmap = dijkstra_map.get_or_insert_with(|| {
            DijkstraMap::new(map.width, map.height, &search_targets, map, 20.0)
        });
        if let Some(destination) = DijkstraMap::find_lowest_exit(&dmap, idx, map) {
            let distance = DistanceAlg::Pythagoras.distance2d(*pos, *player_pos);
            let destination = if distance > 1.2 {
                map.index_to_point2d(destination)
            } else {
                *player_pos
            };

            let mut attacked = false;
            positions
                .iter(ecs)
                .filter(|(_, target_pos)| **target_pos == destination)
                .for_each(|(victim, _)| {
                    if ecs
                        .entry_ref(*victim)
                        .unwrap()
                        .get_component::<Player>()
                        .is_ok()
                    {
                        commands.push((
                            (),
                            WantsToAttack {
                                attacker: *entity,
                                victim: *victim,
                            },
                        ));
                        attacked = true;
                    }
                });

            if !attacked {
                commands.push((
                    (),
                    WantsToMove {
                        entity: *entity,
                        destination,
                    },
                ));
            }
        }
    });
}
