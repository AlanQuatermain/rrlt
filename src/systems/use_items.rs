use super::name_for;
use crate::prelude::*;
use legion::query::*;
use legion::storage::Component;
use legion::world::EntryRef;

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
    Eat,
}

#[system]
#[read_component(ActivateItem)]
#[read_component(ProvidesHealing)]
#[write_component(Pools)]
#[read_component(ProvidesDungeonMap)]
#[read_component(Damage)]
#[read_component(Name)]
#[read_component(Player)]
#[read_component(Point)]
#[read_component(Consumable)]
#[read_component(Item)]
#[read_component(AreaOfEffect)]
#[read_component(Confusion)]
#[read_component(Faction)]
#[read_component(Equippable)]
#[read_component(Equipped)]
#[read_component(ProvidesFood)]
#[read_component(TownPortal)]
#[write_component(HungerClock)]
#[read_component(ObfuscatedName)]
#[read_component(MagicItem)]
#[write_component(IdentifiedItem)]
pub fn use_items(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &mut Map,
    #[resource] gamelog: &mut Gamelog,
    #[resource] turn_state: &mut TurnState,
    #[resource] particle_builder: &mut ParticleBuilder,
    #[resource] dm: &mut MasterDungeonMap,
) {
    let mut operations = Vec::new();
    <(Entity, &ActivateItem)>::query().for_each(ecs, |(entity, activate)| {
        let item = ecs.entry_ref(activate.item);
        if let Ok(item) = item {
            let mut used_item = false;
            if let Ok(healing) = item.get_component::<ProvidesHealing>() {
                let targets = if let Some(target) = activate.target {
                    find_targets::<Pools>(ecs, &item, &target, map)
                } else {
                    vec![activate.used_by]
                };

                operations.extend(targets.iter().map(|target| Operation {
                    command: Command::Heal {
                        amount: healing.amount,
                    },
                    user: activate.used_by,
                    item: activate.item,
                    target: *target,
                }));
                used_item = true;
            }

            if item.get_component::<ProvidesDungeonMap>().is_ok() {
                // map.revealed_tiles.iter_mut().for_each(|t| *t = true);
                gamelog
                    .entries
                    .push("The map is revealed to you!".to_string());
                used_item = true;
                *turn_state = TurnState::RevealMap { row: 0 };
            }

            if item.get_component::<TownPortal>().is_ok() {
                if map.depth == 0 {
                    gamelog.entries.push(
                        "You are already in the town, so the scroll has no effect.".to_string(),
                    );
                } else {
                    used_item = true;
                    *turn_state = TurnState::TownPortal;
                }
            }

            if let Ok(equippable) = item.get_component::<Equippable>() {
                operations.push(Operation {
                    command: Command::Equip {
                        slot: equippable.slot,
                    },
                    user: activate.used_by,
                    item: activate.item,
                    target: activate.used_by,
                });
                used_item = true;
            }

            if item.get_component::<ProvidesFood>().is_ok() {
                operations.push(Operation {
                    command: Command::Eat,
                    user: activate.used_by,
                    item: activate.item,
                    target: activate.used_by,
                });
                used_item = true;
            }

            if let Some(target) = activate.target {
                if let Ok(damage) = item.get_component::<Damage>() {
                    let targets = find_all_targets(ecs, &item, &target, map);
                    operations.extend(targets.iter().map(|target| Operation {
                        command: Command::Damage { amount: damage.0 },
                        user: activate.used_by,
                        item: activate.item,
                        target: *target,
                    }));
                    used_item = true;
                }
                if let Ok(confusion) = item.get_component::<Confusion>() {
                    let targets = find_targets::<Pools>(ecs, &item, &target, map);
                    operations.extend(targets.iter().map(|target| Operation {
                        command: Command::Confuse {
                            duration: confusion.0,
                        },
                        user: activate.used_by,
                        item: activate.item,
                        target: *target,
                    }));
                    used_item = true;
                }
            }

            if used_item && item.get_component::<Consumable>().is_ok() {
                // remove the item
                commands.add_component(activate.item, Consumed {});
                commands.add_component(activate.used_by, EquipmentChanged);
            }

            if used_item
                && item.get_component::<MagicItem>().is_ok()
                && item.get_component::<ObfuscatedName>().is_ok()
            {
                // Self-identify the item
                if let Ok(name) = item.get_component::<Name>() {
                    commands.add_component(activate.item, IdentifiedItem(name.0.clone()));
                }
            }
        }

        // remove the use-item command
        commands.remove(*entity);
    });

    operations
        .iter()
        .for_each(|operation| match operation.command {
            Command::Heal { amount } => {
                if let Some(logs) = apply_healing(
                    ecs,
                    operation.item,
                    operation.target,
                    operation.user,
                    amount,
                    particle_builder,
                ) {
                    gamelog.entries.extend(logs);
                }
            }
            Command::Damage { amount } => {
                commands.push((
                    (),
                    InflictDamage {
                        target: operation.target,
                        user_entity: operation.user,
                        damage: amount,
                        item_entity: Some(operation.item),
                    },
                ));
            }
            Command::Confuse { duration } => {
                if let Some(logs) =
                    apply_confusion(ecs, operation.target, commands, duration, particle_builder)
                {
                    gamelog.entries.extend(logs);
                }
            }
            Command::Equip { slot } => {
                if let Some(logs) =
                    equip_item(ecs, operation.item, operation.target, commands, slot)
                {
                    gamelog.entries.extend(logs);
                }
            }
            Command::Eat => {
                if let Some(logs) = eat(ecs, operation.item, operation.user) {
                    gamelog.entries.extend(logs);
                }
            }
        });
}

fn find_targets<T: Component>(
    ecs: &SubWorld,
    item: &EntryRef,
    target: &Point,
    map: &Map,
) -> Vec<Entity> {
    let mut entities: Vec<Entity> = Vec::new();

    if let Ok(area_of_effect) = item.get_component::<AreaOfEffect>() {
        // Area target -- can aim anywhere
        let blast_tiles = field_of_view_set(*target, area_of_effect.0, map);
        <(Entity, &Point)>::query()
            .filter(component::<T>())
            .iter(ecs)
            .filter(|(_, p)| blast_tiles.contains(*p))
            .for_each(|(e, _)| entities.push(*e));
    } else {
        // Single target -- must have one valid target
        <(Entity, &Point)>::query()
            .filter(component::<T>())
            .iter(ecs)
            .filter(|(_, p)| **p == *target)
            .for_each(|(e, _)| entities.push(*e));
    }

    entities
}

fn find_all_targets(ecs: &SubWorld, item: &EntryRef, target: &Point, map: &Map) -> Vec<Entity> {
    let mut entities: Vec<Entity> = Vec::new();

    if let Ok(area_of_effect) = item.get_component::<AreaOfEffect>() {
        // Area target -- can aim anywhere
        let blast_tiles = field_of_view_set(*target, area_of_effect.0, map);
        <(Entity, &Point)>::query()
            .iter(ecs)
            .filter(|(_, p)| blast_tiles.contains(*p))
            .for_each(|(e, _)| entities.push(*e));
    } else {
        // Single target -- must have one valid target
        <(Entity, &Point)>::query()
            .iter(ecs)
            .filter(|(_, p)| **p == *target)
            .for_each(|(e, _)| entities.push(*e));
    }

    entities
}

fn apply_healing(
    ecs: &mut SubWorld,
    item_entity: Entity,
    target_entity: Entity,
    user_entity: Entity,
    amount: i32,
    particle_builder: &mut ParticleBuilder,
) -> Option<Vec<String>> {
    let user_name = name_for(&user_entity, ecs);
    let target_name = name_for(&target_entity, ecs);
    let item_name = name_for(&item_entity, ecs).0;

    if let Ok(mut target) = ecs.entry_mut(target_entity) {
        if let Ok(pos) = target.get_component::<Point>() {
            particle_builder.request(*pos, ColorPair::new(GREEN, BLACK), to_cp437('♥'), 200.0);
        }
        if let Ok(stats) = target.get_component_mut::<Pools>() {
            let amount_healed = i32::min(amount, stats.hit_points.max - stats.hit_points.current);
            stats.hit_points.current += amount_healed;

            let log_entry = if user_name.1 {
                if target_name.1 {
                    format!("You used {}, healing {} hp.", item_name, amount_healed)
                } else {
                    format!(
                        "You used {} on {}, healing {} hp.",
                        item_name, target_name.0, amount_healed
                    )
                }
            } else {
                if target_name.1 {
                    format!(
                        "{} used {} on you, healing {} hp.",
                        user_name.0, item_name, amount_healed
                    )
                } else if user_entity == target_entity {
                    format!(
                        "{} used {}, healing {} hp.",
                        user_name.0, item_name, amount_healed
                    )
                } else {
                    format!(
                        "{} used {} on {}, healing {} hp.",
                        user_name.0, item_name, target_name.0, amount_healed
                    )
                }
            };
            return Some(vec![log_entry]);
        }
    }
    return None;
}

fn apply_confusion(
    ecs: &mut SubWorld,
    target_entity: Entity,
    commands: &mut CommandBuffer,
    duration: i32,
    particle_builder: &mut ParticleBuilder,
) -> Option<Vec<String>> {
    let target_name = name_for(&target_entity, ecs);

    if let Ok(target) = ecs.entry_ref(target_entity) {
        commands.add_component(target_entity, Confusion(duration));
        let log = if target_name.1 {
            format!("You are confused!")
        } else {
            format!("{} is confused!", target_name.0)
        };
        if let Ok(pos) = target.get_component::<Point>() {
            particle_builder.request(*pos, ColorPair::new(MAGENTA, BLACK), to_cp437('?'), 200.0);
        }
        return Some(vec![log]);
    }
    None
}

fn equip_item(
    ecs: &mut SubWorld,
    item_entity: Entity,
    target_entity: Entity,
    commands: &mut CommandBuffer,
    slot: EquipmentSlot,
) -> Option<Vec<String>> {
    let target_name = name_for(&target_entity, ecs);
    let user_name = if target_name.1 {
        "You".to_string()
    } else {
        target_name.0
    };
    let item_name = name_for(&item_entity, ecs).0;

    if let Ok(item) = ecs.entry_ref(item_entity) {
        if item.get_component::<Equipped>().is_ok() {
            // unequip and leave
            commands.remove_component::<Equipped>(item_entity);
            return Some(vec![format!("{} unequipped {}.", user_name, item_name)]);
        }
    }

    if ecs.entry_ref(target_entity).is_ok() {
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
        commands.add_component(
            item_entity,
            Equipped {
                owner: target_entity,
                slot: slot,
            },
        );
        logs.push(format!("{} equipped {}.", user_name, item_name));
        return Some(logs);
    }
    None
}

fn eat(ecs: &mut SubWorld, item_entity: Entity, user_entity: Entity) -> Option<Vec<String>> {
    let item_name = name_for(&item_entity, ecs).0;
    let user_name = name_for(&user_entity, ecs);

    if let Ok(mut user) = ecs.entry_mut(user_entity) {
        if let Ok(mut clock) = user.get_component_mut::<HungerClock>() {
            clock.state = HungerState::WellFed;
            clock.duration = 20;
            let log_line = if user_name.1 {
                format!("You eat the {}.", item_name)
            } else {
                format!("{} eats the {}.", user_name.0, item_name)
            };
            return Some(vec![log_line]);
        }
    }
    None
}
