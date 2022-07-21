use crate::{prelude::*, KeyState};

#[system]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(Attackable)]
#[write_component(Pools)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(MeleeWeapon)]
#[read_component(Ranged)]
#[write_component(FieldOfView)]
#[read_component(HungerClock)]
#[write_component(Door)]
#[read_component(BlocksVisibility)]
#[read_component(BlocksTile)]
#[write_component(Render)]
#[read_component(Bystander)]
pub fn player_input(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &Map,
    #[resource] gamelog: &mut Gamelog,
    #[resource] key_state: &mut KeyState,
    #[resource] turn_state: &mut TurnState,
) {
    // don't process input here if we're in inventory mode.
    if *turn_state != TurnState::AwaitingInput {
        return;
    }
    let mut players = <(Entity, &Point)>::query().filter(component::<Player>());

    if let Some(key) = key_state.key {
        if key_state.shift {
            let hotkey = match key {
                VirtualKeyCode::Key1 => Some(0),
                VirtualKeyCode::Key2 => Some(1),
                VirtualKeyCode::Key3 => Some(2),
                VirtualKeyCode::Key4 => Some(3),
                VirtualKeyCode::Key5 => Some(4),
                VirtualKeyCode::Key6 => Some(5),
                VirtualKeyCode::Key7 => Some(6),
                VirtualKeyCode::Key8 => Some(7),
                VirtualKeyCode::Key9 => Some(8),
                _ => None,
            };
            if let Some(hotkey) = hotkey {
                *turn_state = use_consumable_hotkey(ecs, commands, hotkey);
                key_state.key = None;
                return;
            }
        }

        let mut waiting = false;
        let delta = match key {
            VirtualKeyCode::LShift
            | VirtualKeyCode::RShift
            | VirtualKeyCode::LControl
            | VirtualKeyCode::RControl
            | VirtualKeyCode::LAlt
            | VirtualKeyCode::RAlt
            | VirtualKeyCode::LWin
            | VirtualKeyCode::RWin => {
                // don't use a turn when user only pressed a meta-key
                return;
            }
            VirtualKeyCode::Left => Point::new(-1, 0),
            VirtualKeyCode::Right => Point::new(1, 0),
            VirtualKeyCode::Up => Point::new(0, -1),
            VirtualKeyCode::Down => Point::new(0, 1),
            VirtualKeyCode::Q => Point::new(-1, -1),
            VirtualKeyCode::F => Point::new(1, -1),
            VirtualKeyCode::Z => Point::new(-1, 1),
            VirtualKeyCode::C => Point::new(1, 1),
            VirtualKeyCode::Period => {
                if key_state.shift {
                    // Player typed '>'
                    let (_player, player_pos) = players
                        .iter(ecs)
                        .find_map(|(entity, pos)| Some((*entity, *pos)))
                        .unwrap();
                    let player_idx = map.point2d_to_index(player_pos);
                    if map.tiles[player_idx] == TileType::DownStairs {
                        *turn_state = TurnState::NextLevel;
                        key_state.key = None;
                        return;
                    }

                    gamelog
                        .entries
                        .push("There is no way down from here.".to_string());
                } else {
                    waiting = true;
                }
                Point::zero()
            }
            VirtualKeyCode::Comma => {
                let (player, player_pos) = players
                    .iter(ecs)
                    .find_map(|(entity, pos)| Some((*entity, *pos)))
                    .unwrap();

                if key_state.shift {
                    // Player typed '<'
                    let player_idx = map.point2d_to_index(player_pos);
                    if map.tiles[player_idx] == TileType::UpStairs {
                        *turn_state = TurnState::PreviousLevel;
                        key_state.key = None;
                        return;
                    }

                    gamelog
                        .entries
                        .push("There is no way up from here.".to_string());
                } else {
                    let mut items = <(Entity, &Item, &Point)>::query();
                    items
                        .iter(ecs)
                        .filter(|(_, _, &item_pos)| item_pos == player_pos)
                        .for_each(|(entity, _, _)| {
                            commands.push((
                                (),
                                WantsToCollect {
                                    who: player,
                                    what: *entity,
                                },
                            ));
                        });
                }
                Point::zero()
            }
            VirtualKeyCode::I => {
                *turn_state = if *turn_state != TurnState::ShowingInventory {
                    TurnState::ShowingInventory
                } else {
                    TurnState::AwaitingInput
                };
                key_state.key = None;
                return;
            }
            VirtualKeyCode::D => {
                *turn_state = if *turn_state != TurnState::ShowingDropItems {
                    TurnState::ShowingDropItems
                } else {
                    TurnState::AwaitingInput
                };
                key_state.key = None;
                return;
            }
            VirtualKeyCode::Escape => {
                *turn_state = TurnState::SaveGame;
                key_state.key = None;
                return;
            }
            _ => Point::zero(),
        };

        let (player_entity, player_pos, destination) = players
            .iter(ecs)
            .find_map(|(entity, pos)| Some((*entity, *pos, *pos + delta)))
            .unwrap();

        let mut enemies = <(Entity, &Point)>::query().filter(component::<Attackable>());
        let mut bystanders = <(Entity, &Point)>::query().filter(component::<Bystander>());
        if delta != Point::zero() {
            let mut attacked = false;
            let mut opened = false;
            enemies
                .iter(ecs)
                .filter(|(_, pos)| **pos == destination)
                .for_each(|(entity, _)| {
                    attacked = true;
                    commands.push((
                        (),
                        WantsToAttack {
                            attacker: player_entity,
                            victim: *entity,
                        },
                    ));
                });

            if !attacked {
                <(Entity, &Point, &mut Door, &mut Render)>::query()
                    .filter(component::<BlocksTile>())
                    .iter_mut(ecs)
                    .filter(|(_, pos, _, _)| **pos == destination)
                    .for_each(|(entity, _, door, render)| {
                        door.open = true;
                        render.glyph = to_cp437('/');
                        commands.remove_component::<BlocksVisibility>(*entity);
                        commands.remove_component::<BlocksTile>(*entity);
                        opened = true;
                    });
                if opened {
                    // mark fov as dirty
                    <&mut FieldOfView>::query()
                        .filter(component::<Player>())
                        .for_each_mut(ecs, |fov| fov.is_dirty = true);
                }
            }

            if !attacked && !opened {
                // If destination isn't walkable, don't eat the turn.
                let idx = map.point2d_to_index(destination);
                if !map.tiles[idx].is_walkable() {
                    return;
                }

                commands.push((
                    (),
                    WantsToMove {
                        entity: player_entity,
                        destination,
                    },
                ));

                // Are we displacing an NPC?
                bystanders
                    .iter(ecs)
                    .filter(|(_, pos)| **pos == destination)
                    .for_each(|(entity, _)| {
                        commands.push((
                            (),
                            WantsToMove {
                                entity: *entity,
                                destination: player_pos,
                            },
                        ));
                    })
            }
        } else if waiting {
            // Player is standing still.
            // If well fed, we may heal.
            let hunger_state = <&HungerClock>::query()
                .filter(component::<Player>())
                .iter(ecs)
                .nth(0)
                .map(|clock| clock.state);

            // If no monsters are visible, heal 1 hp.
            let fov = <&FieldOfView>::query()
                .filter(component::<Player>())
                .iter(ecs)
                .nth(0)
                .unwrap();

            let num_enemies = <&Point>::query()
                .filter(component::<Attackable>())
                .iter(ecs)
                .filter(|pos| fov.visible_tiles.contains(pos))
                .count();

            let can_heal = num_enemies == 0
                && match hunger_state {
                    Some(HungerState::WellFed) => true,
                    Some(HungerState::Normal) => true,
                    _ => false,
                };
            if can_heal {
                <&mut Pools>::query()
                    .filter(component::<Player>())
                    .for_each_mut(ecs, |stats| {
                        if stats.hit_points.current < stats.hit_points.max {
                            stats.hit_points.current += 1;
                        }
                    });
            }
        }
        *turn_state = TurnState::PlayerTurn;
        key_state.key = None;
    }
}

fn use_consumable_hotkey(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    hotkey: i32,
) -> TurnState {
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();
    let mut carried_consumables = Vec::new();
    <(Entity, &Carried)>::query()
        .filter(component::<Consumable>())
        .iter(ecs)
        .filter(|(_, c)| c.0 == *player_entity)
        .for_each(|(e, _)| carried_consumables.push(*e));

    if (hotkey as usize) < carried_consumables.len() {
        if let Ok(entry) = ecs.entry_ref(carried_consumables[hotkey as usize]) {
            if let Ok(ranged) = entry.get_component::<Ranged>() {
                return TurnState::RangedTargeting {
                    range: ranged.0,
                    item: carried_consumables[hotkey as usize],
                };
            }

            // Otherwise, register intent to use
            commands.push((
                (),
                ActivateItem {
                    used_by: *player_entity,
                    item: carried_consumables[hotkey as usize],
                    target: None,
                },
            ));
        }
    }

    TurnState::PlayerTurn
}
