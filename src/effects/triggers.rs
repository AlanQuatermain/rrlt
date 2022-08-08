use crate::prelude::*;

pub fn item_trigger(
    creator: Option<Entity>,
    item: Entity,
    targets: &Targets,
    ecs: &mut SubWorld,
    particle_builder: &mut ParticleBuilder,
    turn_state: &mut TurnState,
    map: &Map,
    commands: &mut CommandBuffer,
) {
    let mut entry = ecs.entry_mut(item).unwrap();
    if let Ok(c) = entry.get_component_mut::<Consumable>() {
        if c.charges < 1 {
            let name = entry.get_component::<Name>().unwrap();
            crate::gamelog::Logger::new()
                .append(&name.0)
                .append("is out of charges!")
                .log();
            return;
        } else {
            c.charges -= 1;
        }
    }
    std::mem::drop(entry);

    // use the item via generic system
    let did_something = event_trigger(
        creator,
        item,
        targets,
        ecs,
        particle_builder,
        turn_state,
        map,
        commands,
    );

    // If it was a consumable, then it gets deleted.
    if did_something {
        if let Ok(entry) = ecs.entry_ref(item) {
            if let Ok(consumable) = entry.get_component::<Consumable>() {
                if consumable.max_charges == 0 {
                    commands.remove(item);
                }
            }
        }
    }
}

pub fn spell_trigger(
    creator: Option<Entity>,
    spell: Entity,
    targets: &Targets,
    ecs: &mut SubWorld,
    particle_builder: &mut ParticleBuilder,
    turn_state: &mut TurnState,
    map: &Map,
    commands: &mut CommandBuffer,
) {
    let template = ecs
        .entry_ref(spell)
        .unwrap()
        .get_component::<SpellTemplate>()
        .unwrap()
        .clone();

    let mut targeting = targets.clone();

    let (self_destruct, targets_self, aoe) = <(
        Entity,
        Option<&SingleActivation>,
        Option<&AlwaysTargetsSelf>,
        Option<&AreaOfEffect>,
    )>::query()
    .iter(ecs)
    .filter(|(e, _, _, _)| **e == spell)
    .find_map(|(_, act, trg, aoe)| Some((act.is_some(), trg.is_some(), aoe.map(|x| *x))))
    .unwrap();

    let mut cast_ok = false;
    if let Some(caster) = creator {
        let pos = ecs
            .entry_ref(caster)
            .unwrap()
            .get_component::<Point>()
            .ok()
            .map(|p| *p)
            .unwrap_or(Point::zero())
            .clone();

        if let Ok(stats) = ecs.entry_mut(caster).unwrap().get_component_mut::<Pools>() {
            if template.mana_cost <= stats.mana.current {
                stats.mana.current -= template.mana_cost;
                cast_ok = true;

                if targets_self {
                    targeting = if let Some(aoe) = aoe {
                        Targets::Tiles {
                            tiles: aoe_tiles(map, pos, aoe.0),
                        }
                    } else {
                        Targets::Tile {
                            tile_idx: map.point2d_to_index(pos),
                        }
                    };
                }
            }
        }
    }

    if cast_ok {
        event_trigger(
            creator,
            spell,
            &targeting,
            ecs,
            particle_builder,
            turn_state,
            map,
            commands,
        );
    }

    if self_destruct && creator.is_some() {
        // remove all hit points
        let mut entry = ecs.entry_mut(creator.unwrap()).unwrap();
        if let Ok(stats) = entry.get_component_mut::<Pools>() {
            stats.hit_points.current = 0;
        }
        // don't trigger on-death if it self-destructed
        commands.remove_component::<OnDeath>(creator.unwrap());
    }
}

fn event_trigger(
    creator: Option<Entity>,
    item: Entity,
    targets: &Targets,
    ecs: &mut SubWorld,
    _particle_builder: &mut ParticleBuilder,
    turn_state: &mut TurnState,
    map: &Map,
    _commands: &mut CommandBuffer,
) -> bool {
    let entry = ecs.entry_ref(item).unwrap();
    let mut did_something = false;

    // Providing food
    if entry.get_component::<ProvidesFood>().is_ok() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        did_something = true;
        if let Ok(name) = entry.get_component::<Name>() {
            crate::gamelog::Logger::new()
                .append("You eat the")
                .append(&name.0)
                .log();
        }
    }

    // Magic mapper
    if entry.get_component::<ProvidesDungeonMap>().is_ok() {
        *turn_state = TurnState::RevealMap { row: 0 };
        did_something = true;
        crate::gamelog::Logger::new()
            .append("The map is revealed to you!")
            .log();
    }

    // Town Portal
    if entry.get_component::<TownPortal>().is_ok() {
        if map.depth == 0 {
            crate::gamelog::Logger::new()
                .append("You are already in town, so the scroll does nothing.")
                .log();
        } else {
            crate::gamelog::Logger::new()
                .append("You are teleported back to town!")
                .log();
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
    if entry.get_component::<Confusion>().is_ok() {
        if let Ok(duration) = entry.get_component::<Duration>() {
            add_effect(
                creator,
                EffectType::Confusion { turns: duration.0 },
                targets.clone(),
            );
            did_something = true;
        }
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

    // Remove Curse
    if entry.get_component::<ProvidesRemoveCurse>().is_ok() {
        *turn_state = TurnState::ShowingRemoveCurse;
        did_something = true;
    }

    // Identify Scroll
    if entry.get_component::<ProvidesIdentify>().is_ok() {
        *turn_state = TurnState::ShowingIdentify;
        did_something = true;
    }

    // Attribute Modifiers
    if let Ok(attr) = entry.get_component::<AttributeBonus>() {
        if let Ok(name) = entry.get_component::<Name>() {
            add_effect(
                creator,
                EffectType::AttributeEffect {
                    bonus: attr.clone(),
                    name: name.0.clone(),
                    duration: 10,
                },
                targets.clone(),
            );
            did_something = true;
        }
    }

    // Restore Mana
    if let Ok(mana) = entry.get_component::<ProvidesMana>() {
        add_effect(
            creator,
            EffectType::Mana { amount: mana.0 },
            targets.clone(),
        );
        did_something = true;
    }

    // Teach spell
    if let Ok(spell) = entry.get_component::<TeachSpell>() {
        let name = &spell.0;
        add_effect(
            creator,
            EffectType::LearnSpell {
                name: name.clone(),
                spell: find_spell_entity(ecs, &name).unwrap(),
            },
            targets.clone(),
        );
        did_something = true;
    }

    // Slow / Haste
    if let Ok(slow) = entry.get_component::<Slow>() {
        add_effect(
            creator,
            EffectType::Slow {
                initiative_penalty: slow.initiative_penalty,
            },
            targets.clone(),
        );
        did_something = true;
    }

    // Ongoing Damage
    if let Ok(ongoing) = entry.get_component::<DamageOverTime>() {
        add_effect(
            creator,
            EffectType::DamageOverTime {
                damage: ongoing.damage,
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
        if let Some(start_pos) = find_item_position(ecs, item, creator, map) {
            match targets {
                Targets::Tile { tile_idx } => {
                    spawn_line_particles(ecs, start_pos, *tile_idx, part, map)
                }
                Targets::Tiles { tiles } => tiles.iter().for_each(|tile_idx| {
                    spawn_line_particles(ecs, start_pos, *tile_idx, part, map)
                }),
                Targets::Single { target } => {
                    if let Some(end_pos) = entity_position(ecs, *target, map) {
                        spawn_line_particles(ecs, start_pos, end_pos[0], part, map);
                    }
                }
                Targets::Area { targets } => {
                    targets.iter().for_each(|target| {
                        if let Some(end_pos) = entity_position(ecs, *target, map) {
                            spawn_line_particles(ecs, start_pos, end_pos[0], part, map);
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

pub fn learn_spell(
    ecs: &mut SubWorld,
    effect: &EffectSpawner,
    name: String,
    spell: Entity,
    _commands: &mut CommandBuffer,
) {
    if effect.creator.is_none() {
        return;
    }
    let spell_template = <(Entity, &SpellTemplate)>::query()
        .iter(ecs)
        .find_map(|(e, s)| if *e == spell { Some(s.clone()) } else { None })
        .unwrap();

    let mut entry = ecs.entry_mut(effect.creator.unwrap()).unwrap();
    if let Ok(known) = entry.get_component_mut::<KnownSpells>() {
        let already_known = known
            .spells
            .iter()
            .filter(|s| s.display_name == name)
            .count()
            != 0;
        if !already_known {
            known.spells.push(KnownSpell {
                display_name: name.clone(),
                mana_cost: spell_template.mana_cost,
            });
        }
    }
}

fn spawn_line_particles(
    _ecs: &SubWorld,
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
