use super::name_for;
use crate::prelude::*;

#[system(for_each)]
#[read_component(InflictDamage)]
#[read_component(Point)]
#[write_component(Pools)]
#[read_component(Attributes)]
#[read_component(Name)]
#[read_component(Player)]
#[read_component(SingleActivation)]
pub fn damage(
    message: &Entity,
    command: &InflictDamage,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &mut Map,
    #[resource] particle_builder: &mut ParticleBuilder,
) {
    let user_name = name_for(&command.user_entity, ecs);
    let target_name = name_for(&command.target, ecs);
    let item_name = command.item_entity.map(|item| name_for(&item, ecs).0);

    let mut xp_gain = 0;
    let mut gold_gain = 0.0f32;

    if let Ok(mut target) = ecs.entry_mut(command.target) {
        let mut target_idx = 0usize;
        if let Ok(pos) = target.get_component::<Point>() {
            // Only leave bloodstains when something hurts itself with an item
            // E.g. not when damaged due to hunger.
            if command.item_entity.is_some() || command.user_entity != command.target {
                map.bloodstains.insert(map.point2d_to_index(*pos));
            }
            particle_builder.request(*pos, ColorPair::new(RED, BLACK), to_cp437('‼'), 200.0);
            target_idx = map.point2d_to_index(*pos);
        }
        if let Ok(stats) = target.get_component_mut::<Pools>() {
            if !stats.god_mode {
                let amount = i32::min(command.damage, stats.hit_points.current);
                stats.hit_points.current -= amount;
                if stats.hit_points.current < 1 {
                    crate::spatial::remove_entity(command.target, target_idx);
                }

                if let Some(item_name) = item_name {
                    log_for_item_damage(&user_name, &target_name, &item_name, amount)
                } else if command.user_entity == command.target {
                    log_for_self_damage(&user_name, amount)
                } else {
                    log_for_damage(&user_name, &target_name, amount)
                }
            }

            if user_name.1 && !target_name.1 && stats.hit_points.current <= 0 {
                xp_gain += stats.level * 100;
                gold_gain += stats.gold;
            }
        } else if target.get_component::<Item>().is_ok() {
            // destroy the item outright
            crate::spatial::remove_entity(command.target, target_idx);
            commands.remove(command.target);
            log_for_destroyed_item(&user_name, &target_name.0);
        }
    };

    if let Ok(user) = ecs.entry_ref(command.user_entity) {
        if user.get_component::<SingleActivation>().is_ok() {
            // one-shot trap, etc. Remove from game now.
            commands.remove(command.user_entity);
        }
    }

    if xp_gain != 0 || gold_gain != 0.0 {
        award_xp_and_gold(
            ecs,
            particle_builder,
            &command.user_entity,
            xp_gain,
            gold_gain,
        );
    }

    commands.remove(*message);
}

fn award_xp_and_gold(
    ecs: &mut SubWorld,
    particle_builder: &mut ParticleBuilder,
    entity: &Entity,
    xp_gain: i32,
    gold_gain: f32,
) {
    <(Entity, &mut Pools, &Attributes, &Point)>::query()
        .filter(component::<Player>())
        .iter_mut(ecs)
        .filter(|(e, _, _, _)| *e == entity)
        .for_each(|(_, stats, attrs, pos)| {
            stats.xp += xp_gain;
            stats.gold += gold_gain;
            let goal = stats.level * 1000;
            if stats.xp >= goal {
                // Gained a level!
                stats.level += 1;
                crate::gamelog::Logger::new()
                    .append("Congratulations, you are now level")
                    .append(format!("{}", stats.level))
                    .log();

                stats.hit_points.max =
                    player_hp_at_level(attrs.fitness.base + attrs.fitness.modifiers, stats.level);
                stats.hit_points.current = stats.hit_points.max;
                stats.mana.max = mana_at_level(
                    attrs.intelligence.base + attrs.intelligence.modifiers,
                    stats.level,
                );
                stats.mana.current = stats.mana.max;

                for i in 0..10 {
                    if pos.y - i > 1 {
                        particle_builder.request(
                            *pos - Point::new(0, i),
                            ColorPair::new(GOLD, BLACK),
                            to_cp437('░'),
                            300.0,
                        );
                    }
                }
            }
        });
}

fn log_for_damage(user_name: &(String, bool), target_name: &(String, bool), amount: i32) {
    if user_name.1 {
        crate::gamelog::Logger::new()
            .append("You hit")
            .npc_name(&target_name.0)
            .append("causing")
            .damage(amount)
            .append("hp damage.")
            .log();
    } else if target_name.1 {
        crate::gamelog::Logger::new()
            .npc_name(&user_name.0)
            .append("hits you, causing")
            .damage(amount)
            .append("hp damage.")
            .log();
    } else {
        crate::gamelog::Logger::new()
            .npc_name(&user_name.0)
            .append("hits")
            .npc_name(&target_name.0)
            .append("causing")
            .damage(amount)
            .append("hp damage.")
            .log();
    }
}

fn log_for_self_damage(user_name: &(String, bool), amount: i32) {
    if user_name.1 {
        crate::gamelog::Logger::new()
            .append("You take")
            .damage(amount)
            .append("hp damage.")
            .log();
    } else {
        crate::gamelog::Logger::new()
            .npc_name(&user_name.0)
            .append("takes")
            .damage(amount)
            .append("hp damage.")
            .log();
    }
}

fn log_for_item_damage(
    user_name: &(String, bool),
    target_name: &(String, bool),
    item_name: &String,
    amount: i32,
) {
    if user_name.1 {
        if target_name.1 {
            crate::gamelog::Logger::new()
                .append("You inflicted")
                .damage(amount)
                .append("hp damage on yourself with")
                .item_name(item_name)
                .log();
        } else {
            crate::gamelog::Logger::new()
                .append("You used")
                .item_name(item_name)
                .append("on")
                .npc_name(&target_name.0)
                .append("inflicting")
                .damage(amount)
                .append("hp damage.")
                .log();
        }
    } else if target_name.1 {
        crate::gamelog::Logger::new()
            .npc_name(&user_name.0)
            .append("used")
            .item_name(item_name)
            .append("inflicting")
            .damage(amount)
            .append("hp damage on you!")
            .log();
    } else {
        crate::gamelog::Logger::new()
            .npc_name(&user_name.0)
            .append("used")
            .item_name(item_name)
            .append("on")
            .npc_name(&target_name.0)
            .append("inflicting")
            .damage(amount)
            .append("damage.")
            .log();
    }
}

fn log_for_destroyed_item(user_name: &(String, bool), item_name: &String) {
    if user_name.1 {
        crate::gamelog::Logger::new()
            .append("You destroyed")
            .item_name(item_name)
            .log();
    } else {
        crate::gamelog::Logger::new()
            .npc_name(&user_name.0)
            .append("destroyed")
            .item_name(item_name)
            .log();
    }
}
