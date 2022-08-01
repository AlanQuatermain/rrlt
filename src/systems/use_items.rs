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
    Eat,
}

#[system(for_each)]
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
#[filter(!component::<Equippable>())]
pub fn use_items(
    entity: &Entity,
    use_item: &UseItem,
    aoe: Option<&AreaOfEffect>,
    ecs: &mut SubWorld,
    #[resource] map: &mut Map,
    commands: &mut CommandBuffer,
) {
    commands.remove_component::<UseItem>(*entity);

    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();

    if use_item.user == *player_entity {
        add_effect(
            Some(*player_entity),
            EffectType::Identify,
            Targets::Single { target: *entity },
        );
    }

    // Call into the effects system
    add_effect(
        Some(use_item.user),
        EffectType::ItemUse { item: *entity },
        match use_item.target {
            None => Targets::Single {
                target: *player_entity,
            },
            Some(target) => {
                if let Some(aoe) = aoe {
                    Targets::Tiles {
                        tiles: aoe_tiles(&*map, target, aoe.0),
                    }
                } else {
                    Targets::Tile {
                        tile_idx: map.point2d_to_index(target),
                    }
                }
            }
        },
    );
}

#[system(for_each)]
#[read_component(Item)]
#[read_component(Carried)]
#[write_component(Equipped)]
#[read_component(Equippable)]
#[read_component(Name)]
#[read_component(MagicItem)]
#[read_component(UseItem)]
#[read_component(Player)]
pub fn equip(
    entity: &Entity,
    name: &Name,
    equippable: &Equippable,
    carried: &Carried,
    use_item: &UseItem,
    equipped: Option<&Equipped>,
    magic: Option<&MagicItem>,
    #[resource] log: &mut Gamelog,
    #[resource] dm: &MasterDungeonMap,
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
) {
    commands.remove_component::<UseItem>(*entity);

    let target_name = name_for(&use_item.user, ecs);
    let user_name = if target_name.1 {
        "You".to_string()
    } else {
        target_name.0
    };

    if equipped.is_some() {
        // already equipped, so unequip
        commands.remove_component::<Equipped>(*entity);
        log.entries
            .push(format!("{} unequipped {}.", user_name, &name.0));
        return;
    }

    // Equip the item
    let target_slot = equippable.slot;

    // Remove anything already in the slot
    <(Entity, &Equipped, &Name)>::query()
        .filter(component::<Item>())
        .iter(ecs)
        .filter(|(_, e, _)| e.owner == use_item.user && e.slot == target_slot)
        .for_each(|(e, _, n)| {
            commands.remove_component::<Equipped>(*e);
            log.entries
                .push(format!("{} unequipped {}.", user_name, &n.0));
        });

    // Assign this to the slot
    commands.add_component(
        *entity,
        Equipped {
            owner: carried.0,
            slot: target_slot,
        },
    );
    log.entries
        .push(format!("{} equipped {}.", user_name, &name.0));

    // auto-identify if it's magic
    if magic.is_some() && !dm.identified_items.contains(&name.0) {
        println!("Identifying item");
        add_effect(
            Some(use_item.user),
            EffectType::Identify,
            Targets::Single { target: *entity },
        );
    }
}

////////////////////////////////////////////////////////////

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
