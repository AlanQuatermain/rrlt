use super::name_for;
use crate::prelude::*;

#[system(for_each)]
#[read_component(InflictDamage)]
#[read_component(Point)]
#[write_component(Pools)]
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
    #[resource] gamelog: &mut Gamelog,
) {
    let user_name = name_for(&command.user_entity, ecs);
    let target_name = name_for(&command.target, ecs);
    let item_name = command.item_entity.map(|item| name_for(&item, ecs).0);

    if let Ok(mut target) = ecs.entry_mut(command.target) {
        if let Ok(pos) = target.get_component::<Point>() {
            // Only leave bloodstains when something hurts itself with an item
            // E.g. not when damaged due to hunger.
            if command.item_entity.is_some() || command.user_entity != command.target {
                map.bloodstains.insert(map.point2d_to_index(*pos));
            }
            particle_builder.request(*pos, ColorPair::new(RED, BLACK), to_cp437('‼'), 200.0);
        }
        if let Ok(stats) = target.get_component_mut::<Pools>() {
            let amount = i32::min(command.damage, stats.hit_points.current);
            stats.hit_points.current -= amount;

            let log_line = if let Some(item_name) = item_name {
                log_for_item_damage(&user_name, &target_name, &item_name, amount)
            } else if command.user_entity == command.target {
                log_for_self_damage(&user_name, amount)
            } else {
                log_for_damage(&user_name, &target_name, amount)
            };
            gamelog.entries.push(log_line);
        } else if target.get_component::<Item>().is_ok() {
            // destroy the item outright
            commands.remove(command.target);
            gamelog
                .entries
                .push(log_for_destroyed_item(&user_name, &target_name.0));
        }
    };

    if let Ok(user) = ecs.entry_ref(command.user_entity) {
        if user.get_component::<SingleActivation>().is_ok() {
            // one-shot trap, etc. Remove from game now.
            commands.remove(command.user_entity);
        }
    }

    commands.remove(*message);
}

fn log_for_damage(user_name: &(String, bool), target_name: &(String, bool), amount: i32) -> String {
    if user_name.1 {
        format!("You hit {}, causing {} damage.", target_name.0, amount)
    } else if target_name.1 {
        format!("{} hits you, causing {} damage.", user_name.0, amount)
    } else {
        format!(
            "{} hits {}, causing {} damage.",
            user_name.0, target_name.0, amount
        )
    }
}

fn log_for_self_damage(user_name: &(String, bool), amount: i32) -> String {
    if user_name.1 {
        format!("You take {} damage!", amount)
    } else {
        format!("{} takes {} damage.", user_name.0, amount)
    }
}

fn log_for_item_damage(
    user_name: &(String, bool),
    target_name: &(String, bool),
    item_name: &String,
    amount: i32,
) -> String {
    if user_name.1 {
        if target_name.1 {
            format!(
                "You inflicted {} damage on yourself with {}!",
                amount, item_name
            )
        } else {
            format!(
                "You used {} on {}, inflicting {} damage.",
                item_name, target_name.0, amount
            )
        }
    } else if target_name.1 {
        format!(
            "{} used {}, inflicting {} damage on you!",
            user_name.0, item_name, amount
        )
    } else {
        format!(
            "{} used {} on {}, inflicting {} damage.",
            user_name.0, item_name, target_name.0, amount
        )
    }
}

fn log_for_destroyed_item(user_name: &(String, bool), item_name: &String) -> String {
    if user_name.1 {
        format!("You destroyed {}!", item_name)
    } else {
        format!("{} destroyed {}!", user_name.0, item_name)
    }
}
