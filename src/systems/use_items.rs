use super::name_for;
use crate::prelude::*;
use legion::query::*;

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
#[read_component(SpellTemplate)]
#[read_component(WantsToCastSpell)]
#[read_component(Name)]
#[read_component(AreaOfEffect)]
#[read_component(Player)]
pub fn spellcasting(
    entity: &Entity,
    wants_cast: &WantsToCastSpell,
    #[resource] map: &Map,
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
) {
    commands.remove_component::<WantsToCastSpell>(*entity);

    let spell = ecs.entry_ref(wants_cast.spell).unwrap();

    // Call into the effects system
    add_effect(
        Some(*entity),
        EffectType::CastSpell {
            spell: wants_cast.spell,
        },
        match wants_cast.target {
            None => Targets::Single { target: *entity },
            Some(target) => {
                if let Ok(aoe) = spell.get_component::<AreaOfEffect>() {
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
    cursed: Option<&CursedItem>,
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
        // already equipped, so unequip -- if we can
        if cursed.is_none() {
            commands.remove_component::<Equipped>(*entity);
            crate::gamelog::Logger::new()
                .append(&user_name)
                .append("unequipped")
                .color(CYAN)
                .append(&name.0)
                .log();
        } else {
            crate::gamelog::Logger::new()
                .append("You cannot unequip")
                .color(CYAN)
                .append(&name.0)
                .color(WHITE)
                .append(" - it is cursed!")
                .log();
        }
        return;
    }

    // Equip the item
    let target_slot = equippable.slot;

    // Remove anything already in the slot
    let mut equip_blocked = false;
    <(Entity, &Equipped, &Name, Option<&CursedItem>)>::query()
        .filter(component::<Item>())
        .iter(ecs)
        .filter(|(_, e, _, _)| e.owner == use_item.user && e.slot == target_slot)
        .for_each(|(e, _, n, c)| {
            if c.is_none() {
                commands.remove_component::<Equipped>(*e);
                crate::gamelog::Logger::new()
                    .append(&user_name)
                    .append("unequipped")
                    .color(CYAN)
                    .append(&n.0)
                    .log();
            } else {
                crate::gamelog::Logger::new()
                    .append("You cannot unequip")
                    .color(CYAN)
                    .append(&name.0)
                    .color(WHITE)
                    .append(" - it is cursed!")
                    .log();
                equip_blocked = true;
            }
        });

    if equip_blocked {
        return;
    }

    // Assign this to the slot
    commands.add_component(
        *entity,
        Equipped {
            owner: carried.0,
            slot: target_slot,
        },
    );
    crate::gamelog::Logger::new()
        .append(&user_name)
        .append("equipped")
        .color(CYAN)
        .append(&name.0)
        .log();
    commands.add_component(use_item.user, EquipmentChanged);

    // auto-identify if it's magic
    if magic.is_some() && !dm.identified_items.contains(&name.0) {
        add_effect(
            Some(use_item.user),
            EffectType::Identify,
            Targets::Single { target: *entity },
        );
    }
}
