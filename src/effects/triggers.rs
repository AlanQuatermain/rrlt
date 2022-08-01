use crate::prelude::*;

use super::particles;

pub fn item_trigger(
    creator: Option<Entity>,
    item: Entity,
    targets: &Targets,
    ecs: &mut SubWorld,
    gamelog: &mut Gamelog,
    particle_builder: &mut ParticleBuilder,
    turn_state: &mut TurnState,
    map: &Map,
    commands: &mut CommandBuffer,
) {
    // use the item via generic system
    let did_something = event_trigger(
        creator,
        item,
        targets,
        ecs,
        gamelog,
        particle_builder,
        turn_state,
        map,
        commands,
    );

    // If it was a consumable, then it gets deleted.
    if did_something {
        if let Ok(entry) = ecs.entry_ref(item) {
            if entry.get_component::<Consumable>().is_ok() {
                commands.remove(item);
            }
        }
    }
}

fn event_trigger(
    creator: Option<Entity>,
    item: Entity,
    targets: &Targets,
    ecs: &mut SubWorld,
    gamelog: &mut Gamelog,
    particle_builder: &mut ParticleBuilder,
    turn_state: &mut TurnState,
    map: &Map,
    commands: &mut CommandBuffer,
) -> bool {
    let entry = ecs.entry_ref(item).unwrap();
    let mut did_something = false;

    // Providing food
    if entry.get_component::<ProvidesFood>().is_ok() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        did_something = true;
        if let Ok(name) = entry.get_component::<Name>() {
            gamelog.entries.push(format!("You eat the {}.", &name.0));
        }
    }

    // Magic mapper
    if entry.get_component::<ProvidesDungeonMap>().is_ok() {
        *turn_state = TurnState::RevealMap { row: 0 };
        did_something = true;
        gamelog
            .entries
            .push("The map is revealed to you!".to_string());
    }

    // Town Portal
    if entry.get_component::<TownPortal>().is_ok() {
        if map.depth == 0 {
            gamelog
                .entries
                .push("You are already in town, so the scroll does nothing.".to_string());
        } else {
            gamelog
                .entries
                .push("You are teleported back to town!".to_string());
            *turn_state = TurnState::TownPortal;
            did_something = true;
        }
    }

    // Healing
    if let Ok(healing) = entry.get_component::<ProvidesHealing>() {
        add_effect(
            creator,
            EffectType::Healing {
                amount: healing.amount,
            },
            targets.clone(),
        );
        did_something = true;
    }

    // Damage
    if let Ok(damage) = entry.get_component::<Damage>() {
        add_effect(
            creator,
            EffectType::Damage { amount: damage.0 },
            targets.clone(),
        );
        did_something = true;
    }

    // Confusion
    if let Ok(confusion) = entry.get_component::<Confusion>() {
        add_effect(
            creator,
            EffectType::Confusion { turns: confusion.0 },
            targets.clone(),
        );
        did_something = true;
    }

    // Teleport
    if let Ok(teleport) = entry.get_component::<TeleportTo>() {
        add_effect(
            creator,
            EffectType::TeleportTo {
                pos: teleport.position,
                depth: teleport.depth,
                player_only: teleport.player_only,
            },
            targets.clone(),
        );
        did_something = true;
    }

    // Simple particle spawn
    if let Ok(part) = entry.get_component::<SpawnParticleBurst>() {
        add_effect(
            creator,
            EffectType::Particle {
                glyph: part.glyph,
                color: ColorPair::new(part.color, RGB::named(BLACK)),
                lifespan: part.lifetime_ms,
            },
            targets.clone(),
        );
    }

    // Line particle spawn
    if let Ok(part) = entry.get_component::<SpawnParticleLine>() {
        if let Some(start_pos) = find_item_position(ecs, item, map) {
            match targets {
                Targets::Tile { tile_idx } => {
                    spawn_line_particles(ecs, start_pos, *tile_idx, part, map)
                }
                Targets::Tiles { tiles } => tiles.iter().for_each(|tile_idx| {
                    spawn_line_particles(ecs, start_pos, *tile_idx, part, map)
                }),
                Targets::Single { target } => {
                    if let Some(end_pos) = entity_position(ecs, *target, map) {
                        spawn_line_particles(ecs, start_pos, end_pos, part, map);
                    }
                }
                Targets::Area { targets } => {
                    targets.iter().for_each(|target| {
                        if let Some(end_pos) = entity_position(ecs, *target, map) {
                            spawn_line_particles(ecs, start_pos, end_pos, part, map);
                        }
                    });
                }
            }
        }
    }

    did_something
}

pub fn trigger(
    creator: Option<Entity>,
    trigger: Entity,
    targets: &Targets,
    ecs: &mut SubWorld,
    gamelog: &mut Gamelog,
    particle_builder: &mut ParticleBuilder,
    turn_state: &mut TurnState,
    map: &Map,
    commands: &mut CommandBuffer,
) {
    commands.remove_component::<Hidden>(trigger);

    // Use via the generic item system
    let did_something = event_trigger(
        creator,
        trigger,
        targets,
        ecs,
        gamelog,
        particle_builder,
        turn_state,
        map,
        commands,
    );

    if did_something
        && ecs
            .entry_ref(trigger)
            .unwrap()
            .get_component::<SingleActivation>()
            .is_ok()
    {
        commands.remove(trigger);
    }
}

fn spawn_line_particles(
    ecs: &SubWorld,
    start: usize,
    end: usize,
    part: &SpawnParticleLine,
    map: &Map,
) {
    let start_pt = map.index_to_point2d(start);
    let end_pt = map.index_to_point2d(end);
    let line = line2d_bresenham(start_pt, end_pt);
    for pt in line.iter() {
        let idx = map.point2d_to_index(*pt);
        add_effect(
            None,
            EffectType::Particle {
                glyph: part.glyph,
                color: ColorPair::new(part.color, RGB::named(BLACK)),
                lifespan: part.lifetime_ms,
            },
            Targets::Tile { tile_idx: idx },
        );
    }
}
