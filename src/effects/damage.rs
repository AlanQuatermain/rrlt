use crate::prelude::*;

pub fn inflict_damage(ecs: &mut SubWorld, damage: &EffectSpawner, _map: &Map, target: Entity) {
    let attacker_name = damage.creator.map(|c| name_for(&c, ecs).0);
    let target_name = name_for(&target, ecs).0;
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .map(|e| *e)
        .unwrap();
    let attacker_is_player = damage.creator.map(|c| c == player_entity).unwrap_or(false);
    let target_is_player = target == player_entity;

    if let Ok(mut entry) = ecs.entry_mut(target) {
        if let Ok(mut stats) = entry.get_component_mut::<Pools>() {
            if !stats.god_mode {
                if let Some(creator) = damage.creator {
                    if creator == target {
                        // Avoid hurting yourself
                        return;
                    }
                }
                if let EffectType::Damage { amount } = damage.effect_type {
                    if let Some(attacker_name) = attacker_name {
                        crate::gamelog::Logger::new()
                            .npc_name(&attacker_name)
                            .append("hits")
                            .npc_name(&target_name)
                            .append("for")
                            .damage(amount)
                            .append("hp.")
                            .log();
                    } else {
                        crate::gamelog::Logger::new()
                            .npc_name(&target_name)
                            .append("is hit for")
                            .damage(amount)
                            .append("hp.")
                            .log();
                    }

                    stats.hit_points.current -= amount;
                    if stats.hit_points.current < 1 {
                        add_effect(
                            damage.creator,
                            EffectType::EntityDeath,
                            Targets::Single { target },
                        );
                    }
                    add_effect(None, EffectType::Bloodstain, Targets::Single { target });
                    add_effect(
                        None,
                        EffectType::Particle {
                            glyph: to_cp437('‼'),
                            color: ColorPair::new(ORANGE, BLACK),
                            lifespan: 200.0,
                        },
                        Targets::Single { target },
                    );

                    if target_is_player {
                        crate::gamelog::record_event("Damage Taken", amount);
                    }
                    if attacker_is_player {
                        crate::gamelog::record_event("Damage Inflicted", amount);
                    }
                }
            }
        }
    }
}

pub fn bloodstain(map: &mut Map, indices: Vec<usize>) {
    for tile_idx in indices {
        map.bloodstains.insert(tile_idx);
    }
}

pub fn heal_damage(ecs: &mut SubWorld, heal: &EffectSpawner, target: Entity) {
    let mut entry = ecs.entry_mut(target).unwrap();
    if let Ok(stats) = entry.get_component_mut::<Pools>() {
        if let EffectType::Healing { amount } = heal.effect_type {
            stats.hit_points.current =
                i32::min(stats.hit_points.max, stats.hit_points.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: to_cp437('‼'),
                    color: ColorPair::new(GREEN, BLACK),
                    lifespan: 200.0,
                },
                Targets::Single { target },
            );
        }
    }
}

pub fn restore_mana(ecs: &mut SubWorld, mana: &EffectSpawner, target: Entity) {
    let mut entry = ecs.entry_mut(target).unwrap();
    if let Ok(stats) = entry.get_component_mut::<Pools>() {
        if let EffectType::Mana { amount } = mana.effect_type {
            stats.mana.current = i32::min(stats.mana.max, stats.mana.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: to_cp437('‼'),
                    color: ColorPair::new(BLUE, BLACK),
                    lifespan: 200.0,
                },
                Targets::Single { target },
            );
        }
    }
}

pub fn add_confusion(
    _ecs: &SubWorld,
    effect: &EffectSpawner,
    target: Entity,
    commands: &mut CommandBuffer,
) {
    if let EffectType::Confusion { turns } = &effect.effect_type {
        commands.push((
            StatusEffect { target },
            Confusion,
            Duration(*turns),
            Name("Confusion".to_string()),
            SerializeMe,
        ));
    }
}

pub fn attribute_effect(
    _ecs: &mut SubWorld,
    effect: &EffectSpawner,
    target: Entity,
    commands: &mut CommandBuffer,
) {
    if let EffectType::AttributeEffect {
        bonus,
        name,
        duration,
    } = &effect.effect_type
    {
        commands.push((
            StatusEffect { target },
            bonus.clone(),
            Duration(*duration),
            Name(name.clone()),
            SerializeMe,
        ));
        commands.add_component(target, EquipmentChanged);
    }
}

pub fn slow(
    _ecs: &mut SubWorld,
    effect: &EffectSpawner,
    target: Entity,
    commands: &mut CommandBuffer,
) {
    if let EffectType::Slow { initiative_penalty } = &effect.effect_type {
        commands.push((
            StatusEffect { target },
            Slow {
                initiative_penalty: *initiative_penalty,
            },
            Duration(5),
            if *initiative_penalty > 0.0 {
                Name("Slowed".to_string())
            } else {
                Name("Hasted".to_string())
            },
            SerializeMe,
        ));
        commands.add_component(target, EquipmentChanged);
    }
}

pub fn damage_over_time(
    _ecs: &mut SubWorld,
    effect: &EffectSpawner,
    target: Entity,
    commands: &mut CommandBuffer,
) {
    if let EffectType::DamageOverTime { damage } = &effect.effect_type {
        commands.push((
            StatusEffect { target },
            DamageOverTime { damage: *damage },
            Duration(5),
            Name("Ongoing Damage".to_string()),
            SerializeMe,
        ));
    }
}

pub fn death(ecs: &mut SubWorld, effect: &EffectSpawner, target: Entity, map: &Map) {
    let mut xp_gain = 0;
    let mut gold_gain = 0.0f32;

    if let Some(idxes) = entity_position(ecs, target, map) {
        for idx in idxes {
            crate::spatial::remove_entity(target, idx);
        }
    }

    if let Some(source) = effect.creator {
        let player_entity = <Entity>::query()
            .filter(component::<Player>())
            .iter(ecs)
            .nth(0)
            .unwrap();
        if source == *player_entity {
            if let Ok(target_entry) = ecs.entry_mut(target) {
                if let Ok(target_stats) = target_entry.get_component::<Pools>() {
                    xp_gain += target_stats.level * 100;
                    gold_gain += target_stats.gold;
                }
            }

            if xp_gain != 0 || gold_gain != 0.0 {
                <(&mut Attributes, &mut Pools, &mut Skills, &Point)>::query()
                    .filter(component::<Player>())
                    .for_each_mut(ecs, |(attrs, stats, skills, pos)| {
                        stats.xp += xp_gain;
                        stats.gold += gold_gain;
                        if stats.xp >= stats.level * 1000 {
                            // We've gone up a level!
                            stats.xp -= stats.level * 1000;
                            stats.level += 1;
                            crate::gamelog::Logger::new()
                                .color(MAGENTA)
                                .append("Congratulations, you are now level")
                                .append(format!("{}", stats.level))
                                .log();

                            // Improve a random attribute
                            let mut rng = RandomNumberGenerator::new();
                            match rng.roll_dice(1, 4) {
                                1 => {
                                    attrs.might.base += 1;
                                    crate::gamelog::Logger::new()
                                        .color(GREEN)
                                        .append("You feel stronger!")
                                        .log();
                                }
                                2 => {
                                    attrs.fitness.base += 1;
                                    crate::gamelog::Logger::new()
                                        .color(GREEN)
                                        .append("You feel healthier!")
                                        .log();
                                }
                                3 => {
                                    attrs.quickness.base += 1;
                                    crate::gamelog::Logger::new()
                                        .color(GREEN)
                                        .append("You feel quicker!")
                                        .log();
                                }
                                _ => {
                                    attrs.intelligence.base += 1;
                                    crate::gamelog::Logger::new()
                                        .color(GREEN)
                                        .append("You feel smarter!")
                                        .log();
                                }
                            }

                            // Improve all skills
                            for skill in skills.0.iter_mut() {
                                *skill.1 += 1;
                            }

                            stats.hit_points.max = player_hp_at_level(
                                attrs.fitness.base + attrs.fitness.modifiers,
                                stats.level,
                            );
                            stats.hit_points.current = stats.hit_points.max;
                            stats.mana.max = mana_at_level(
                                attrs.intelligence.base + attrs.intelligence.modifiers,
                                stats.level,
                            );
                            stats.mana.current = stats.mana.max;

                            for i in 0..10 {
                                if pos.y - i > 1 {
                                    add_effect(
                                        None,
                                        EffectType::Particle {
                                            glyph: to_cp437('░'),
                                            color: ColorPair::new(GOLD, BLACK),
                                            lifespan: 400.0,
                                        },
                                        Targets::Tile {
                                            tile_idx: map.point2d_to_index(*pos - Point::new(0, i)),
                                        },
                                    );
                                }
                            }
                        }
                    });
            }
        }
    }
}
