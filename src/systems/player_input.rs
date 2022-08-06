use std::cmp::Ordering;

use crate::{prelude::*, KeyState};

#[system]
#[write_component(Point)]
#[read_component(Player)]
#[read_component(Faction)]
#[write_component(Pools)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Weapon)]
#[read_component(Ranged)]
#[write_component(FieldOfView)]
#[read_component(HungerClock)]
#[write_component(Door)]
#[read_component(BlocksVisibility)]
#[read_component(BlocksTile)]
#[write_component(Render)]
#[read_component(Hidden)]
#[read_component(Name)]
#[read_component(Vendor)]
#[read_component(KnownSpells)]
#[read_component(TileSize)]
#[read_component(StatusEffect)]
#[read_component(Confusion)]
#[read_component(Target)]
#[read_component(Equipped)]
pub fn player_input(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &mut Map,
    #[resource] gamelog: &mut Gamelog,
    #[resource] key_state: &mut KeyState,
    #[resource] turn_state: &mut TurnState,
    #[resource] rng: &mut RandomNumberGenerator,
) {
    // don't process input here if we're in inventory mode.
    if *turn_state != TurnState::AwaitingInput {
        return;
    }

    if let Some(key) = key_state.key {
        if key_state.shift || key_state.control {
            if let Some(hotkey) = get_hotkey(key_state) {
                if key_state.shift {
                    *turn_state = use_consumable_hotkey(ecs, commands, hotkey);
                } else if key_state.control {
                    *turn_state = use_spell_hotkey(ecs, commands, hotkey, gamelog);
                }
                key_state.key = None;
                return;
            }
        }

        let (player_entity, player_pos, player_fov) = <(Entity, &Point, &FieldOfView)>::query()
            .filter(component::<Player>())
            .iter(ecs)
            .find_map(|(entity, pos, fov)| Some((*entity, *pos, fov.clone())))
            .unwrap();

        // Is the player confused?
        let confused = <&StatusEffect>::query()
            .filter(component::<Confusion>())
            .iter(ecs)
            .filter(|st| st.target == player_entity)
            .count()
            != 0;
        if confused {
            // eat the key-press, spawn particle, miss turn.
            add_effect(
                None,
                EffectType::Particle {
                    glyph: to_cp437('?'),
                    color: ColorPair::new(CYAN, BLACK),
                    lifespan: 400.0,
                },
                Targets::Single {
                    target: player_entity,
                },
            );
            *turn_state = TurnState::Ticking;
            return;
        }

        let mut delta = Point::zero();
        match process_key_input(key, key_state) {
            KeyInputResponse::DoNothing => return,
            KeyInputResponse::Move { delta: mv } => delta = mv,
            KeyInputResponse::Collect => {
                try_collect_items(ecs, player_entity, player_pos, commands);
                *turn_state = TurnState::Ticking;
            }
            KeyInputResponse::ShowCheatMenu => *turn_state = TurnState::ShowCheatMenu,
            KeyInputResponse::ShowDropMenu => *turn_state = TurnState::ShowingDropItems,
            KeyInputResponse::ShowInventory => *turn_state = TurnState::ShowingInventory,
            KeyInputResponse::UpStairs => try_climb_stairs(map, turn_state, player_pos, gamelog),
            KeyInputResponse::DownStairs => {
                try_descend_stairs(map, turn_state, player_pos, gamelog)
            }
            KeyInputResponse::StandStill => {
                try_wait_player(ecs, rng);
                *turn_state = TurnState::Ticking;
            }
            KeyInputResponse::SaveGame => *turn_state = TurnState::SaveGame,
            KeyInputResponse::CycleTargets => {
                cycle_targets(ecs, player_entity, &player_pos, &player_fov, map, commands);
                *turn_state = TurnState::AwaitingInput;
            }
            KeyInputResponse::FireRangedWeapon => {
                // There must be a target somewhere
                if let Some(target) = <Entity>::query()
                    .filter(component::<Target>())
                    .iter(ecs)
                    .nth(0)
                    .map(|e| *e)
                {
                    commands.add_component(player_entity, WantsToShoot { target });
                    *turn_state = TurnState::Ticking;
                } else {
                    *turn_state = TurnState::AwaitingInput;
                }
            }
        }

        if delta != Point::zero() {
            let destination = player_pos + delta;
            match try_move_player(player_entity, player_pos, destination, map, ecs, commands) {
                MoveResult::Moved => {
                    *turn_state = TurnState::Ticking;
                }
                MoveResult::Stood | MoveResult::OpenedDoor => {
                    *turn_state = TurnState::Ticking;
                }
                MoveResult::Attack { entity } => {
                    commands.push((
                        (),
                        WantsToAttack {
                            attacker: player_entity,
                            victim: entity,
                        },
                    ));
                    *turn_state = TurnState::Ticking;
                }
                MoveResult::OpenShop { entity } => {
                    *turn_state = TurnState::ShowingVendor {
                        vendor: entity,
                        mode: VendorMode::Buy,
                    }
                }
            }
        }
        key_state.key = None;
    }
}

fn cycle_targets(
    ecs: &mut SubWorld,
    player: Entity,
    player_pos: &Point,
    player_fov: &FieldOfView,
    map: &Map,
    commands: &mut CommandBuffer,
) {
    let possible_targets = get_player_target_list(ecs, player, player_pos, player_fov, map);
    let current_target = <Entity>::query()
        .filter(component::<Target>())
        .iter(ecs)
        .nth(0)
        .map(|e| *e);

    <Entity>::query()
        .filter(component::<Target>())
        .for_each(ecs, |e| commands.remove_component::<Target>(*e));

    if let Some(current_target) = current_target {
        if possible_targets.len() > 1 {
            let mut index = 0usize;
            for (i, target) in possible_targets.iter().enumerate() {
                if target.1 == current_target {
                    index = i;
                }
            }

            if index > possible_targets.len() - 2 {
                commands.add_component(possible_targets[0].1, Target);
            } else {
                commands.add_component(possible_targets[index + 1].1, Target);
            }
        }
    }
}

fn get_player_target_list(
    ecs: &mut SubWorld,
    player: Entity,
    player_pos: &Point,
    player_fov: &FieldOfView,
    map: &Map,
) -> Vec<(f32, Entity)> {
    let maybe_range = <(&Equipped, &Weapon)>::query()
        .iter(ecs)
        .filter(|(e, w)| e.owner == player && w.range.is_some())
        .nth(0)
        .map(|(_, w)| w.range.unwrap());
    if maybe_range.is_none() {
        return Vec::new();
    }
    let range = maybe_range.unwrap();

    let mut possible_targets: Vec<(f32, Entity, usize)> = Vec::new();

    <(Entity, &Point)>::query()
        .filter(component::<Faction>() & !component::<Player>())
        .for_each(ecs, |(entity, pos)| {
            if player_fov.visible_tiles.contains(pos) {
                let map_idx = map.point2d_to_index(*pos);
                let distance = DistanceAlg::Pythagoras.distance2d(*player_pos, *pos);
                if distance < range as f32 {
                    possible_targets.push((distance, *entity, map_idx));
                }
            }
        });

    possible_targets.sort_by(|a, b| {
        let mut c = a.0.partial_cmp(&b.0).unwrap();
        if c == Ordering::Equal {
            c = a.2.partial_cmp(&b.2).unwrap();
        }
        c
    });
    possible_targets.iter().map(|(a, b, _)| (*a, *b)).collect()
}

#[system(for_each)]
#[read_component(Player)]
#[read_component(Point)]
#[read_component(FieldOfView)]
#[read_component(Target)]
#[read_component(Equipped)]
#[read_component(Weapon)]
#[read_component(Faction)]
#[read_component(Name)]
#[read_component(EntityMoved)]
#[read_component(WantsToMove)]
#[read_component(EquipmentChanged)]
#[filter(component::<Player>())]
pub fn update_targeting(
    entity: &Entity,
    pos: &Point,
    fov: &FieldOfView,
    #[resource] map: &Map,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    let possible_targets = get_player_target_list(ecs, *entity, pos, fov, map);
    if possible_targets.is_empty() {
        // No targets, or not wielding something that can target stuff
        <Entity>::query()
            .filter(component::<Target>())
            .for_each(ecs, |e| commands.remove_component::<Target>(*e));
        return;
    }

    // Clear current targets from all the things
    <Entity>::query()
        .filter(component::<Target>())
        .for_each(ecs, |e| commands.remove_component::<Target>(*e));

    // println!("Targets: {}", possible_targets.len());
    if !possible_targets.is_empty() {
        commands.add_component(possible_targets[0].1, Target);
    }
}

fn get_hotkey(key_state: &KeyState) -> Option<i32> {
    if let Some(key) = key_state.key {
        match key {
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
        }
    } else {
        None
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

            let item = carried_consumables[hotkey as usize];
            commands.add_component(
                item,
                UseItem {
                    user: *player_entity,
                    target: None,
                },
            );
        }
    }

    TurnState::Ticking
}

fn use_spell_hotkey(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    hotkey: i32,
    gamelog: &mut Gamelog,
) -> TurnState {
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap()
        .clone();
    let mut player = ecs.entry_mut(player_entity).unwrap();
    let spells = if let Ok(known) = player.get_component::<KnownSpells>() {
        known.spells.clone()
    } else {
        Vec::new()
    };

    if (hotkey as usize) < spells.len() {
        if let Ok(stats) = player.get_component_mut::<Pools>() {
            let spell = &spells[hotkey as usize];
            if stats.mana.current >= spell.mana_cost {
                if let Some(spell_entity) = find_spell_entity(ecs, &spell.display_name) {
                    let spell_entry = ecs.entry_ref(spell_entity).unwrap();
                    if let Ok(range) = spell_entry.get_component::<Ranged>() {
                        return TurnState::RangedTargeting {
                            range: range.0,
                            item: spell_entity,
                        };
                    }

                    commands.add_component(
                        player_entity,
                        WantsToCastSpell {
                            spell: spell_entity,
                            target: None,
                        },
                    );
                }
            } else {
                gamelog.entries.push(format!(
                    "You don't have enough mana to cast {}",
                    spell.display_name
                ));
            }
        }
    }

    TurnState::Ticking
}

fn swap_entity(
    ecs: &mut SubWorld,
    player_entity: Entity,
    other_entity: Entity,
    from: Point,
    to: Point,
    map: &Map,
    commands: &mut CommandBuffer,
) {
    // Swap positions.
    commands.add_component(player_entity, to);
    <(Entity, &mut Point, &mut FieldOfView)>::query()
        .filter(component::<Player>())
        .iter_mut(ecs)
        .filter(|(e, _, _)| **e == player_entity)
        .for_each(|(_, pos, fov)| {
            *pos = to;
            fov.is_dirty = true;
        });

    <(Entity, &mut Point, &mut FieldOfView)>::query()
        .iter_mut(ecs)
        .filter(|(e, _, _)| **e == other_entity)
        .for_each(|(_, pos, fov)| {
            *pos = from;
            fov.is_dirty = true;
        });

    let from_idx = map.point2d_to_index(from);
    let to_idx = map.point2d_to_index(to);
    crate::spatial::move_entity(player_entity, from_idx, to_idx);
    crate::spatial::move_entity(other_entity, to_idx, from_idx);

    commands.add_component(player_entity, EntityMoved);
    commands.add_component(other_entity, EntityMoved);
}

fn entities_in_tile(ecs: &SubWorld, idx: usize, map: &Map) -> Vec<Entity> {
    let pos = map.index_to_point2d(idx);
    <(Entity, &Point, Option<&TileSize>)>::query()
        .iter(ecs)
        .filter(|(_, p, s)| {
            if let Some(size) = s {
                let rect = Rect::with_size(p.x, p.y, size.x, size.y);
                rect.point_in_rect(pos)
            } else {
                **p == pos
            }
        })
        .map(|(e, _, _)| *e)
        .collect()
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum MoveResult {
    Stood,
    Moved,
    OpenedDoor,
    Attack { entity: Entity },
    OpenShop { entity: Entity },
}

fn try_move_player(
    player_entity: Entity,
    player_pos: Point,
    destination: Point,
    map: &mut Map,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
) -> MoveResult {
    let destination_idx = map.point2d_to_index(destination);

    for entity in entities_in_tile(ecs, destination_idx, map) {
        let entry = ecs.entry_ref(entity).unwrap();

        let reaction: Reaction;
        if let Ok(faction) = entry.get_component::<Faction>() {
            reaction = faction_reaction(&faction.name, "Player", &RAWS.lock().unwrap());
            if reaction == Reaction::Attack {
                return MoveResult::Attack { entity };
            } else if entry.get_component::<Vendor>().is_ok() {
                return MoveResult::OpenShop { entity };
            } else {
                swap_entity(
                    ecs,
                    player_entity,
                    entity,
                    player_pos,
                    destination,
                    map,
                    commands,
                );
                return MoveResult::Moved;
            }
        } else if entry.get_component::<Door>().is_ok() {
            if try_open_door(ecs, entity, destination, commands) {
                return MoveResult::OpenedDoor;
            }
        }
    }

    // If destination isn't walkable, don't eat the turn.
    let destination_idx = map.point2d_to_index(destination);
    if crate::spatial::is_blocked(destination_idx) {
        return MoveResult::Stood;
    }

    // Fire out the movement intent
    commands.add_component(player_entity, WantsToMove { destination });
    MoveResult::Moved
}

fn try_open_door(
    ecs: &mut SubWorld,
    door: Entity,
    location: Point,
    commands: &mut CommandBuffer,
) -> bool {
    let mut opened = false;
    if let Ok(mut entry) = ecs.entry_mut(door) {
        if let Ok(door_info) = entry.get_component_mut::<Door>() {
            if door_info.open {
                // already open, do nothing
                return false;
            }
            door_info.open = true;
            opened = true;
        }
        if let Ok(render) = entry.get_component_mut::<Render>() {
            render.glyph = to_cp437('/');
        }
    }

    if opened {
        commands.remove_component::<BlocksVisibility>(door);
        commands.remove_component::<BlocksTile>(door);

        // Update fields of view near the door
        <&mut FieldOfView>::query()
            .iter_mut(ecs)
            .filter(|fov| fov.visible_tiles.contains(&location))
            .for_each(|fov| fov.is_dirty = true);
    }

    opened
}

fn try_wait_player(ecs: &mut SubWorld, rng: &mut RandomNumberGenerator) {
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

    let num_enemies = <(&Point, &Faction)>::query()
        .iter(ecs)
        .filter(|(pos, faction)| {
            fov.visible_tiles.contains(pos)
                && faction_reaction(&faction.name, "Player", &RAWS.lock().unwrap())
                    == Reaction::Attack
        })
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
                if rng.roll_dice(1, 6) == 1 {
                    stats.mana.current = i32::min(stats.mana.max, stats.mana.current + 1);
                }
            });
    }
}

enum KeyInputResponse {
    DoNothing,
    ShowCheatMenu,
    ShowInventory,
    ShowDropMenu,
    Move { delta: Point },
    StandStill,
    DownStairs,
    UpStairs,
    Collect,
    SaveGame,
    CycleTargets,
    FireRangedWeapon,
}

fn process_key_input(key: VirtualKeyCode, key_state: &mut KeyState) -> KeyInputResponse {
    match key {
        VirtualKeyCode::LShift
        | VirtualKeyCode::RShift
        | VirtualKeyCode::LControl
        | VirtualKeyCode::RControl
        | VirtualKeyCode::LAlt
        | VirtualKeyCode::RAlt
        | VirtualKeyCode::LWin
        | VirtualKeyCode::RWin => {
            // don't use a turn when user only pressed a meta-key
            KeyInputResponse::DoNothing
        }
        VirtualKeyCode::Left => KeyInputResponse::Move {
            delta: Point::new(-1, 0),
        },
        VirtualKeyCode::Right => KeyInputResponse::Move {
            delta: Point::new(1, 0),
        },
        VirtualKeyCode::Up => KeyInputResponse::Move {
            delta: Point::new(0, -1),
        },
        VirtualKeyCode::Down => KeyInputResponse::Move {
            delta: Point::new(0, 1),
        },
        VirtualKeyCode::Q => KeyInputResponse::Move {
            delta: Point::new(-1, -1),
        },
        VirtualKeyCode::F => KeyInputResponse::Move {
            delta: Point::new(1, -1),
        },
        VirtualKeyCode::Z => KeyInputResponse::Move {
            delta: Point::new(-1, 1),
        },
        VirtualKeyCode::C => KeyInputResponse::Move {
            delta: Point::new(1, 1),
        },
        VirtualKeyCode::V => KeyInputResponse::CycleTargets,
        VirtualKeyCode::Backslash => KeyInputResponse::ShowCheatMenu,
        VirtualKeyCode::Period => {
            if key_state.shift {
                KeyInputResponse::DownStairs
            } else {
                KeyInputResponse::StandStill
            }
        }
        VirtualKeyCode::Comma => {
            if key_state.shift {
                KeyInputResponse::UpStairs
            } else {
                KeyInputResponse::Collect
            }
        }
        VirtualKeyCode::I => KeyInputResponse::ShowInventory,
        VirtualKeyCode::D => KeyInputResponse::ShowDropMenu,
        VirtualKeyCode::Escape => KeyInputResponse::SaveGame,
        VirtualKeyCode::Space => KeyInputResponse::FireRangedWeapon,
        _ => KeyInputResponse::DoNothing,
    }
}

fn try_climb_stairs(
    map: &Map,
    turn_state: &mut TurnState,
    player_pos: Point,
    gamelog: &mut Gamelog,
) {
    let player_idx = map.point2d_to_index(player_pos);
    if map.tiles[player_idx] == TileType::UpStairs {
        *turn_state = TurnState::PreviousLevel;
        return;
    }

    gamelog
        .entries
        .push("There is no way up from here.".to_string());
}

fn try_descend_stairs(
    map: &Map,
    turn_state: &mut TurnState,
    player_pos: Point,
    gamelog: &mut Gamelog,
) {
    let player_idx = map.point2d_to_index(player_pos);
    if map.tiles[player_idx] == TileType::DownStairs {
        *turn_state = TurnState::NextLevel;
        return;
    }

    gamelog
        .entries
        .push("There is no way down from here.".to_string());
}

fn try_collect_items(
    ecs: &SubWorld,
    player: Entity,
    player_pos: Point,
    commands: &mut CommandBuffer,
) {
    <(Entity, &Item, &Point)>::query()
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
