use legion::query::*;
use legion::storage::Component;
use legion::world::EntryRef;
use crate::prelude::*;
use super::name_for;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Operation {
    command: Command,
    user: Entity,
    item: Entity,
    target: Entity,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Command {
    Heal { amount: i32 },
    Damage { amount: i32 },
    Confuse { duration: i32 },
    Equip { slot: EquipmentSlot },
}

#[system]
#[read_component(ActivateItem)]
#[read_component(ProvidesHealing)]
#[write_component(Health)]
#[read_component(ProvidesDungeonMap)]
#[read_component(Damage)]
#[read_component(Name)]
#[read_component(Player)]
#[read_component(Point)]
#[read_component(Consumable)]
#[read_component(Item)]
#[read_component(AreaOfEffect)]
#[read_component(Confusion)]
#[read_component(Enemy)]
#[read_component(Equippable)]
#[read_component(Equipped)]
pub fn use_items(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &mut Map,
    #[resource] gamelog: &mut Gamelog
) {
    let mut operations = Vec::new();
    <(Entity, &ActivateItem)>::query()
        .for_each(ecs, |(entity, activate)| {
            let item = ecs.entry_ref(activate.item);
            if let Ok(item) = item {
                let mut used_item = false;
                if let Ok(healing) = item.get_component::<ProvidesHealing>() {
                    let targets = if let Some(target) = activate.target {
                        find_targets::<Health>(ecs, &item, &target, map)
                    } else {
                        vec![activate.used_by]
                    };

                    operations.extend(
                        targets.iter().map(|target| Operation {
                            command: Command::Heal { amount: healing.amount },
                            user: activate.used_by,
                            item: activate.item,
                            target: *target
                        })
                    );
                    used_item = true;
                }

                if item.get_component::<ProvidesDungeonMap>().is_ok() {
                    map.revealed_tiles.iter_mut().for_each(|t| *t = true);
                    used_item = true;
                }

                if let Ok(equippable) = item.get_component::<Equippable>() {
                    operations.push(Operation {
                        command: Command::Equip { slot: equippable.slot },
                        user: activate.used_by,
                        item: activate.item,
                        target: activate.used_by
                    });
                    used_item = true;
                }

                if let Some(target) = activate.target {
                    if let Ok(damage) = item.get_component::<Damage>() {
                        let targets = find_all_targets(ecs, &item, &target, map);
                        operations.extend(
                            targets.iter().map(|target| Operation {
                                command: Command::Damage { amount: damage.0 },
                                user: activate.used_by,
                                item: activate.item,
                                target: *target
                            })
                        );
                        used_item = true;
                    }
                    if let Ok(confusion) = item.get_component::<Confusion>() {
                        let targets = find_targets::<Enemy>(ecs, &item, &target, map);
                        operations.extend(
                            targets.iter().map(|target| Operation {
                                command: Command::Confuse { duration: confusion.0 },
                                user: activate.used_by,
                                item: activate.item,
                                target: *target
                            })
                        );
                        used_item = true;
                    }
                }

                if used_item && item.get_component::<Consumable>().is_ok() {
                    // remove the item
                    commands.remove(activate.item);
                }
            }

            // remove the use-item command
            commands.remove(*entity);
        });

    operations.iter().for_each(|operation| match operation.command {
        Command::Heal{amount} => {
            if let Some(logs) = apply_healing(ecs, operation.item, operation.target, operation.user, amount) {
                gamelog.entries.extend(logs);
            }
        },
        Command::Damage{amount} => {
            if let Some(logs) = apply_damage(ecs, operation.item, operation.target, operation.user, commands, amount) {
                gamelog.entries.extend(logs);
            }
        },
        Command::Confuse{duration} => {
            if let Some(logs) = apply_confusion(ecs, operation.target, commands, duration) {
                gamelog.entries.extend(logs);
            }
        },
        Command::Equip{slot} => {
            if let Some(logs) = equip_item(ecs, operation.item, operation.target, commands, slot) {
                gamelog.entries.extend(logs);
            }
        }
    });
}

fn find_targets<T: Component>(
    ecs: &SubWorld,
    item: &EntryRef,
    target: &Point,
    map: &Map
) -> Vec<Entity> {
    if let Ok(area_of_effect) = item.get_component::<AreaOfEffect>() {
        // Area target -- can aim anywhere
        let mut blast_tiles = field_of_view_set(
            *target, area_of_effect.0, map);
        return <(&Point, Entity)>::query()
            .filter(component::<T>())
            // .filter(component::<Health>())
            .iter(ecs)
            .filter(|(pos, _)| blast_tiles.contains(*pos))
            .map(|(_, mob)| *mob)
            .collect();
    }
    else {
        // Single target -- must have one valid target
        <(&Point, Entity)>::query()
            .filter(component::<T>())
            .iter(ecs)
            .filter(|(pos, _)| *pos == target)
            .map(|(_, mob)| *mob)
            .collect()
    }
}

fn find_all_targets(
    ecs: &SubWorld,
    item: &EntryRef,
    target: &Point,
    map: &Map
) -> Vec<Entity> {
    if let Ok(area_of_effect) = item.get_component::<AreaOfEffect>() {
        // Area target -- can aim anywhere
        let mut blast_tiles = field_of_view_set(
            *target, area_of_effect.0, map);
        return <(&Point, Entity)>::query()
            .iter(ecs)
            .filter(|(pos, _)| blast_tiles.contains(*pos))
            .map(|(_, mob)| *mob)
            .collect();
    }
    else {
        // Single target -- must have one valid target
        <(&Point, Entity)>::query()
            .iter(ecs)
            .filter(|(pos, _)| *pos == target)
            .map(|(_, mob)| *mob)
            .collect()
    }
}

fn apply_healing(
    ecs: &mut SubWorld,
    item_entity: Entity,
    target_entity: Entity,
    user_entity: Entity,
    amount: i32
) -> Option<Vec<String>> {
    let user_name = name_for(&user_entity, ecs);
    let target_name = name_for(&target_entity, ecs);
    let item_name = name_for(&item_entity, ecs).0;

    if let  Ok(mut target) = ecs.entry_mut(target_entity) {
        if let Ok(health) = target.get_component_mut::<Health>() {
            let amount_healed = i32::min(amount, health.max - health.current);
            health.current += amount_healed;

            let log_entry = if user_name.1 {
                if target_name.1 {
                    format!("You used {}, healing {} hp.", item_name, amount_healed)
                }
                else {
                    format!("You used {} on {}, healing {} hp.", item_name, target_name.0, amount_healed)
                }
            }
            else {
                if target_name.1 {
                    format!("{} used {} on you, healing {} hp.", user_name.0, item_name, amount_healed)
                }
                else if user_entity == target_entity {
                    format!("{} used {}, healing {} hp.", user_name.0, item_name, amount_healed)
                }
                else {
                    format!("{} used {} on {}, healing {} hp.", user_name.0, item_name, target_name.0, amount_healed)
                }
            };
            return Some(vec![log_entry]);
        }
    }
    return None
}

fn apply_damage(
    ecs: &mut SubWorld,
    item_entity: Entity,
    target_entity: Entity,
    user_entity: Entity,
    commands: &mut CommandBuffer,
    damage_amount: i32
) -> Option<Vec<String>> {
    let user_name = name_for(&user_entity, ecs);
    let target_name = name_for(&target_entity, ecs);
    let item_name = name_for(&item_entity, ecs).0;

    if let Ok(mut target) = ecs.entry_mut(target_entity) {
        if let Ok(health) = target.get_component_mut::<Health>() {
            let amount = i32::min(damage_amount, health.current);
            health.current -= amount;
            let mut logs = Vec::new();

            if user_name.1 {
                if target_name.1 {
                    logs.push(format!("You inflicted {} damage on yourself with {}!", amount, item_name));
                }
                else {
                    logs.push(format!("You used {} on {}, inflicting {} damage.", item_name, target_name.0, amount));
                }
            }
            else if target_name.1 {
                logs.push(format!("{} used {}, inflicting {} damage on you!", user_name.0, item_name, amount));
            }
            else {
                logs.push(format!("{} used {} on {}, inflicting {} damage.", user_name.0, item_name, target_name.0, amount));
            }

            if health.current <= 0 {
                logs.push(format!("{} is dead!", target_name.0));
                commands.remove(target_entity);
            }

            return Some(logs);
        }
        else if target.get_component::<Item>().is_ok() {
            // destroy the item outright
            commands.remove(target_entity);
            if user_name.1 {
                return Some(vec![format!("You destroyed {}!", target_name.0)]);
            }
            else {
                return Some(vec![format!("{} destroyed {}!", user_name.0, target_name.0)]);
            }
        }
    };
    None
}

fn apply_confusion(
    ecs: &mut SubWorld,
    target_entity: Entity,
    commands: &mut CommandBuffer,
    duration: i32
) -> Option<Vec<String>> {
    let target_name = name_for(&target_entity, ecs);

    if let Ok(target) = ecs.entry_ref(target_entity) {
        if target.get_component::<Enemy>().is_ok() {
            commands.add_component(target_entity, Confusion(duration));
            let log = if target_name.1 {
                format!("You are confused!")
            }
            else {
                format!("{} is confused!", target_name.0)
            };
            return Some(vec![log]);
        }
    }
    None
}

fn equip_item(
    ecs: &mut SubWorld,
    item_entity: Entity,
    target_entity: Entity,
    commands: &mut CommandBuffer,
    slot: EquipmentSlot
) -> Option<Vec<String>> {
    let target_name = name_for(&target_entity, ecs);
    let user_name = if target_name.1 { "You".to_string() } else { target_name.0 };
    let item_name = name_for(&item_entity, ecs).0;

    if let Ok(item) = ecs.entry_ref(item_entity) {
        if item.get_component::<Equipped>().is_ok() {
            // unequip and leave
            commands.remove_component::<Equipped>(item_entity);
            return Some(vec![format!("{} unequipped {}.", user_name, item_name)]);
        }
    }

    if let Ok(target) = ecs.entry_ref(target_entity) {
        let mut logs = Vec::new();
        // Remove anything currently equipped in the same slot
        <(Entity, &Equipped, &Name)>::query()
            .iter_mut(ecs)
            .filter(|(entity, equipped, _)| {
                equipped.owner == target_entity && **entity != item_entity && equipped.slot == slot
            })
            .for_each(|(old_item, _, name)| {
                commands.remove_component::<Equipped>(*old_item);
                logs.push(format!("{} unequipped {}.", user_name, name.0));
            });
        commands.add_component(item_entity, Equipped { owner: target_entity, slot: slot });
        logs.push(format!("{} equipped {}.", user_name, item_name));
        return Some(logs);
    }
    None
}
