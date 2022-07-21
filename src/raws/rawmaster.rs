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
    loot_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws {
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spawn_table: Vec::new(),
                loot_tables: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
            loot_index: HashMap::new(),
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

        self.loot_index = HashMap::new();
        for (i, loot) in self.raws.loot_tables.iter().enumerate() {
            self.loot_index.insert(loot.name.clone(), i);
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

fn find_slot_for_equippable_item(tag: &str, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }

    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if item.weapon.is_some() {
        return EquipmentSlot::Melee;
    } else if let Some(wearable) = &item.wearable {
        return string_to_slot(&wearable.slot);
    }
    panic!("Trying to equip {}, but it has no slot tag.", tag);
}

fn string_to_slot(name: &str) -> EquipmentSlot {
    match name {
        "Shield" => EquipmentSlot::Shield,
        "Head" => EquipmentSlot::Head,
        "Torso" => EquipmentSlot::Torso,
        "Legs" => EquipmentSlot::Legs,
        "Feet" => EquipmentSlot::Feet,
        "Hands" => EquipmentSlot::Hands,
        "Melee" => EquipmentSlot::Melee,
        _ => {
            log(format!("WARNING: Unknown equipment slot type [{}]", name));
            EquipmentSlot::Melee
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
    set_position(&entity, pos, key, raws, commands);

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
        let mut wpn = MeleeWeapon {
            attribute: WeaponAttribute::Might,
            damage_die: weapon.base_damage.clone(),
            hit_bonus: weapon.hit_bonus,
        };
        match weapon.attribute.as_str() {
            "Quickness" => wpn.attribute = WeaponAttribute::Quickness,
            _ => wpn.attribute = WeaponAttribute::Might,
        }
        commands.add_component(entity, wpn);
    }

    if let Some(wearable) = &item_template.wearable {
        let slot = string_to_slot(&wearable.slot);
        commands.add_component(entity, Equippable { slot });
        commands.add_component(
            entity,
            Wearable {
                armor_class: wearable.armor_class,
            },
        );
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

    let entity = commands.push(((), Name(mob_template.name.clone())));
    set_position(&entity, pos, key, raws, commands);

    match mob_template.ai.as_ref() {
        "melee" => {
            commands.add_component(entity, Attackable);
            commands.add_component(entity, ChasingPlayer);
        }
        "bystander" => commands.add_component(entity, Bystander),
        "vendor" => commands.add_component(entity, Vendor),
        "carnivore" => {
            commands.add_component(entity, Attackable);
            commands.add_component(entity, Carnivore);
        }
        "herbivore" => {
            commands.add_component(entity, Attackable);
            commands.add_component(entity, Herbivore);
        }
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
        FieldOfView {
            visible_tiles: HashSet::new(),
            radius: mob_template.vision_range,
            is_dirty: true,
        },
    );

    let mut attr = Attributes::default();
    let mut mob_fitness = 11;
    let mut mob_int = 11;
    if let Some(might) = mob_template.attributes.might {
        attr.might = Attribute {
            base: might,
            modifiers: 0,
            bonus: attr_bonus(might),
        }
    }
    if let Some(fitness) = mob_template.attributes.fitness {
        attr.fitness = Attribute {
            base: fitness,
            modifiers: 0,
            bonus: attr_bonus(fitness),
        };
        mob_fitness = fitness;
    }
    if let Some(quickness) = mob_template.attributes.quickness {
        attr.quickness = Attribute {
            base: quickness,
            modifiers: 0,
            bonus: attr_bonus(quickness),
        }
    }
    if let Some(intelligence) = mob_template.attributes.intelligence {
        attr.intelligence = Attribute {
            base: intelligence,
            modifiers: 0,
            bonus: attr_bonus(intelligence),
        };
        mob_int = intelligence;
    }
    commands.add_component(entity, attr);

    let mob_level = mob_template.level.unwrap_or(1);
    let mob_hp = npc_hp(mob_fitness, mob_level);
    let mob_mana = mana_at_level(mob_int, mob_level);
    commands.add_component(
        entity,
        Pools {
            level: mob_level,
            xp: 0,
            hit_points: Pool {
                current: mob_hp,
                max: mob_hp,
            },
            mana: Pool {
                current: mob_mana,
                max: mob_mana,
            },
        },
    );

    let mut skills = Skills::default();
    if let Some(mobskills) = &mob_template.skills {
        for (name, value) in mobskills.iter() {
            match name.as_ref() {
                "Melee" => {
                    skills.0.insert(Skill::Melee, *value);
                }
                "Defense" => {
                    skills.0.insert(Skill::Defense, *value);
                }
                "Magic" => {
                    skills.0.insert(Skill::Magic, *value);
                }
                _ => {
                    log(format!("Unknown skill referenced: [{}]", name));
                }
            }
        }
    }
    commands.add_component(entity, skills);

    if let Some(wielding) = &mob_template.equipped {
        for tag in wielding.iter() {
            spawn_named_entity(raws, tag, SpawnType::Equipped { by: entity }, commands);
        }
    }

    if let Some(natural) = &mob_template.natural {
        let mut nature = NaturalAttackDefense {
            armor_class: natural.armor_class.unwrap_or(0),
            attacks: Vec::new(),
        };
        if let Some(attacks) = &natural.attacks {
            for nattack in attacks.iter() {
                let attack = NaturalAttack {
                    name: nattack.name.clone(),
                    hit_bonus: nattack.hit_bonus,
                    damage_die: nattack.damage.clone(),
                };
                nature.attacks.push(attack);
            }
        }
        commands.add_component(entity, nature);
    }

    if let Some(loot) = &mob_template.loot_table {
        commands.add_component(entity, LootTable(loot.clone()));
    }

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

    let entity = commands.push(((), Name(template.name.clone())));
    set_position(&entity, pos, key, raws, commands);

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

pub fn get_drop_item(
    raws: &RawMaster,
    rng: &mut RandomNumberGenerator,
    table: &str,
) -> Option<String> {
    raws.loot_index.get(table).map(|idx| {
        let mut rt = RandomTable::new();
        let available_options = &raws.raws.loot_tables[*idx];
        for item in available_options.drops.iter() {
            rt = rt.add(item.name.clone(), item.weight);
        }
        rt.roll(rng)
    })
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

fn set_position(
    entity: &Entity,
    spawn_type: SpawnType,
    tag: &str,
    raws: &RawMaster,
    commands: &mut CommandBuffer,
) {
    match spawn_type {
        SpawnType::AtPosition { point } => commands.add_component(*entity, point),
        SpawnType::Carried { by } => commands.add_component(*entity, Carried(by)),
        SpawnType::Equipped { by } => {
            commands.add_component(*entity, Carried(by));
            commands.add_component(
                *entity,
                Equipped {
                    owner: by,
                    slot: find_slot_for_equippable_item(tag, raws),
                },
            );
        }
    }
}
