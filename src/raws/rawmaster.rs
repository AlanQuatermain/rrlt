use super::Raws;
use crate::components::*;
use crate::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct RawMaster {
    raws: Raws,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws {
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        let mut used_names = HashSet::new();

        self.item_index = HashMap::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                log(format!(
                    "WARNING: duplicate item name in raws [{}]",
                    &item.name
                ))
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }

        self.mob_index = HashMap::new();
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                log(format!(
                    "WARNING: duplicate mob name in raws [{}]",
                    &mob.name
                ))
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }

        self.prop_index = HashMap::new();
        for (i, prop) in self.raws.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                log(format!(
                    "WARNING: duplicate prop name in raws [{}]",
                    &prop.name
                ))
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }

        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                log(format!(
                    "WARNING: Spawn table references unspecified entity [{}]",
                    &spawn.name
                ));
            }
        }
    }
}

pub fn spawn_named_item(
    raws: &RawMaster,
    key: &str,
    pos: SpawnType,
    commands: &mut CommandBuffer,
) -> bool {
    if !raws.item_index.contains_key(key) {
        return false;
    }

    let item_template = &raws.raws.items[raws.item_index[key]];
    let entity = commands.push((crate::components::Item, Name(item_template.name.clone())));

    // Spawn in the specified location
    commands.add_component(entity, get_position(pos));

    if let Some(renderable) = &item_template.renderable {
        commands.add_component(entity, get_renderable(&renderable));
    }

    if let Some(consumable) = &item_template.consumable {
        commands.add_component(entity, Consumable {});
        for effect in consumable.effects.iter() {
            let effect_name = effect.0.as_str();
            match effect_name {
                "provides_healing" => commands.add_component(
                    entity,
                    ProvidesHealing {
                        amount: effect.1.parse::<i32>().unwrap(),
                    },
                ),
                "ranged" => {
                    commands.add_component(entity, Ranged(effect.1.parse::<i32>().unwrap()))
                }
                "damage" => {
                    commands.add_component(entity, Damage(effect.1.parse::<i32>().unwrap()))
                }
                "area_of_effect" => {
                    commands.add_component(entity, AreaOfEffect(effect.1.parse::<i32>().unwrap()))
                }
                "confusion" => {
                    commands.add_component(entity, Confusion(effect.1.parse::<i32>().unwrap()))
                }
                "magic_mapping" => commands.add_component(entity, ProvidesDungeonMap),
                "food" => commands.add_component(entity, ProvidesFood),
                _ => log(format!(
                    "Warning: consumable effect {} not implemented",
                    effect_name
                )),
            }
        }
    }

    if let Some(weapon) = &item_template.weapon {
        commands.add_component(
            entity,
            Equippable {
                slot: EquipmentSlot::Melee,
            },
        );
        commands.add_component(entity, Damage(weapon.power_bonus));
        commands.add_component(entity, Weapon);
    }
    if let Some(shield) = &item_template.shield {
        commands.add_component(
            entity,
            Equippable {
                slot: EquipmentSlot::Shield,
            },
        );
        commands.add_component(entity, Armor(shield.defense_bonus));
    }

    true
}

pub fn spawn_named_mob(
    raws: &RawMaster,
    key: &str,
    pos: SpawnType,
    commands: &mut CommandBuffer,
) -> bool {
    if !raws.mob_index.contains_key(key) {
        return false;
    }
    let mob_template = &raws.raws.mobs[raws.mob_index[key]];

    let entity = commands.push((Name(mob_template.name.clone()), get_position(pos)));

    match mob_template.ai.as_ref() {
        "melee" => {
            commands.add_component(entity, Enemy);
            commands.add_component(entity, ChasingPlayer);
        }
        "bystander" => commands.add_component(entity, Bystander),
        "vendor" => commands.add_component(entity, Vendor),
        _ => {}
    }

    if let Some(quips) = &mob_template.quips {
        commands.add_component(entity, Quips(quips.clone()));
    }

    if let Some(renderable) = &mob_template.renderable {
        commands.add_component(entity, get_renderable(renderable));
    }

    if mob_template.blocks_tile {
        commands.add_component(entity, BlocksTile);
    }
    commands.add_component(
        entity,
        Health {
            max: mob_template.stats.max_hp,
            current: mob_template.stats.hp,
        },
    );
    commands.add_component(entity, Damage(mob_template.stats.power));
    commands.add_component(entity, Armor(mob_template.stats.defense));
    commands.add_component(
        entity,
        FieldOfView {
            visible_tiles: HashSet::new(),
            radius: mob_template.vision_range,
            is_dirty: true,
        },
    );

    true
}

pub fn spawn_named_prop(
    raws: &RawMaster,
    key: &str,
    pos: SpawnType,
    commands: &mut CommandBuffer,
) -> bool {
    if !raws.prop_index.contains_key(key) {
        return false;
    }
    let template = &raws.raws.props[raws.prop_index[key]];

    let entity = commands.push(((), Name(template.name.clone()), get_position(pos)));

    if let Some(renderable) = &template.renderable {
        commands.add_component(entity, get_renderable(renderable));
    }
    if let Some(hidden) = template.hidden {
        if hidden {
            commands.add_component(entity, Hidden);
        }
    }
    if template.blocks_tile.is_some() {
        commands.add_component(entity, BlocksTile);
    }
    if template.blocks_visibility.is_some() {
        commands.add_component(entity, BlocksVisibility {});
    }
    if template.always_visible.is_some() {
        commands.add_component(entity, AlwaysVisible);
    }
    if let Some(door_open) = &template.door_open {
        commands.add_component(entity, Door { open: *door_open });
    }
    if let Some(entry_trigger) = &template.entry_trigger {
        commands.add_component(entity, EntryTrigger);
        for effect in entry_trigger.effects.iter() {
            match effect.0.as_str() {
                "damage" => {
                    commands.add_component(entity, Damage(effect.1.parse::<i32>().unwrap()))
                }
                "single_activation" => commands.add_component(entity, SingleActivation),
                _ => {}
            }
        }
    }

    true
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    key: &str,
    pos: SpawnType,
    commands: &mut CommandBuffer,
) -> bool {
    if raws.item_index.contains_key(key) {
        spawn_named_item(raws, key, pos, commands)
    } else if raws.mob_index.contains_key(key) {
        spawn_named_mob(raws, key, pos, commands)
    } else if raws.prop_index.contains_key(key) {
        spawn_named_prop(raws, key, pos, commands)
    } else {
        false
    }
}

pub fn spawn_table_for_depth(raws: &RawMaster, depth: i32) -> RandomTable {
    use super::spawn_table_structs::SpawnTableEntry;
    let available_options: Vec<&SpawnTableEntry> = raws
        .raws
        .spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = RandomTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt = rt.add(e.name.clone(), weight);
    }
    rt
}

fn get_renderable(renderable: &super::Renderable) -> crate::components::Render {
    Render {
        color: ColorPair::new(
            RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
            RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        ),
        glyph: to_cp437(renderable.glyph.chars().next().unwrap()),
        render_order: renderable.order,
    }
}

fn get_position(spawn_type: SpawnType) -> Point {
    match spawn_type {
        SpawnType::AtPosition { point } => point,
    }
}
