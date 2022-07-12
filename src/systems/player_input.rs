use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(Enemy)]
#[write_component(Health)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Weapon)]
#[read_component(FieldOfView)]
#[read_component(HungerClock)]
pub fn player_input(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &Map,
    #[resource] gamelog: &mut Gamelog,
    #[resource] input_key: &mut Option<VirtualKeyCode>,
    #[resource] turn_state: &mut TurnState
) {
    // don't process input here if we're in inventory mode.
    if *turn_state != TurnState::AwaitingInput {
        return;
    }
    let mut players = <(Entity, &Point)>::query()
        .filter(component::<Player>());

    if let Some(key) = *input_key {
        let mut waiting = false;
        let delta = match key {
            VirtualKeyCode::Left => Point::new(-1, 0),
            VirtualKeyCode::Right => Point::new(1, 0),
            VirtualKeyCode::Up => Point::new(0, -1),
            VirtualKeyCode::Down => Point::new(0, 1),
            VirtualKeyCode::G => {
                let (player, player_pos) = players
                    .iter(ecs)
                    .find_map(|(entity, pos)| Some((*entity, *pos)))
                    .unwrap();
                let mut items = <(Entity, &Item, &Point)>::query();
                items.iter(ecs)
                    .filter(|(_, _, &item_pos)| item_pos == player_pos)
                    .for_each(|(entity, _, _)| {
                        commands.push(((), WantsToCollect {
                            who: player,
                            what: *entity
                        }));
                    });
                Point::zero()
            },
            VirtualKeyCode::Period => {
                let (player, player_pos) = players
                    .iter(ecs)
                    .find_map(|(entity, pos)| Some((*entity, *pos)))
                    .unwrap();
                let player_idx = map.point2d_to_index(player_pos);
                if map.tiles[player_idx] == TileType::DownStairs {
                    *turn_state = TurnState::NextLevel;
                    *input_key = None;
                    return;
                }

                gamelog.entries.push("There is no way down from here.".to_string());
                Point::zero()
            }
            VirtualKeyCode::I => {
                *turn_state = if *turn_state != TurnState::ShowingInventory {
                    TurnState::ShowingInventory
                }
                else {
                    TurnState::AwaitingInput
                };
                *input_key = None;
                return
            },
            VirtualKeyCode::D => {
                *turn_state = if *turn_state != TurnState::ShowingDropItems {
                    TurnState::ShowingDropItems
                }
                else {
                    TurnState::AwaitingInput
                };
                *input_key = None;
                return
            },
            VirtualKeyCode::Escape => {
                *turn_state = TurnState::SaveGame;
                *input_key = None;
                return
            },
            VirtualKeyCode::Space => {
                waiting = true;
                Point::zero()
            },
            _ => Point::zero()
        };
        *input_key = None; // prevent the key being processed twice

        let (player_entity, destination) = players
            .iter(ecs)
            .find_map(|(entity, pos)| Some((*entity, *pos + delta)))
            .unwrap();

        let mut enemies = <(Entity, &Point)>::query().filter(component::<Enemy>());
        if delta != Point::zero() {
            let mut attacked = false;
            enemies
                .iter(ecs)
                .filter(|(_, pos)| **pos == destination)
                .for_each(|(entity, _)| {
                    attacked = true;
                    commands
                        .push(((), WantsToAttack {
                                attacker: player_entity,
                                victim: *entity
                            }));
                });

            if !attacked {
                commands
                    .push(((), WantsToMove {
                        entity: player_entity,
                        destination
                    }));
            }
        }
        else if waiting {
            // Player is standing still.
            // If well fed, we may heal.
            let hunger_state = <&HungerClock>::query()
                .filter(component::<Player>())
                .iter(ecs).nth(0).map(|clock| clock.state);

            // If no monsters are visible, heal 1 hp.
            let fov = <&FieldOfView>::query()
                .filter(component::<Player>())
                .iter(ecs)
                .nth(0)
                .unwrap();

            let num_enemies = <&Point>::query()
                .filter(component::<Enemy>())
                .iter(ecs)
                .filter(|pos| fov.visible_tiles.contains(pos))
                .count();

            let can_heal = num_enemies == 0 && match hunger_state {
                Some(HungerState::WellFed) => true,
                Some(HungerState::Normal) => true,
                _ => false
            };
            if can_heal {
                <&mut Health>::query()
                    .filter(component::<Player>())
                    .for_each_mut(ecs, |health| {
                        if health.current < health.max {
                            health.current += 1;
                        }
                    });
            }
        }
        *turn_state = TurnState::PlayerTurn;
    }
}