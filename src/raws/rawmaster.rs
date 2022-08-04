use super::faction_structs::Reaction;
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
    faction_index: HashMap<String, HashMap<String, Reaction>>,
    spell_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn empty() -> RawMaster {
        RawMaster {
            raws: Raws {
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
                spells: Vec::new(),
                spawn_table: Vec::new(),
                loot_tables: Vec::new(),
                faction_table: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
            loot_index: HashMap::new(),
            faction_index: HashMap::new(),
            spell_index: HashMap::new(),
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

        self.spell_index = HashMap::new();
        for (i, spell) in self.raws.spells.iter().enumerate() {
            self.spell_index.insert(spell.name.clone(), i);
        }

        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                log(format!(
                    "WARNING: Spawn table references unspecified entity [{}]",
                    &spawn.name
                ));
            }
        }

        for faction in self.raws.faction_table.iter() {
            let mut reactions: HashMap<String, Reaction> = HashMap::new();
            for other in faction.responses.iter() {
                reactions.insert(
                    other.0.clone(),
                    match other.1.as_str() {
                        "ignore" => Reaction::Ignore,
                        "flee" => Reaction::Flee,
                        _ => Reaction::Attack,
                    },
                );
            }
            self.faction_index.insert(faction.name.clone(), reactions);
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

fn parse_particle_line(n: &str) -> SpawnParticleLine {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleLine {
        glyph: to_cp437(tokens[0].chars().next().unwrap()),
        color: RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap(),
    }
}

fn parse_particle(n: &str) -> SpawnParticleBurst {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleBurst {
        glyph: to_cp437(tokens[0].chars().next().unwrap()),
        color: RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap(),
    }
}

macro_rules! i32_component {
    ( $name:ident, $item:expr ) => {
        $name($item.1.parse::<i32>().unwrap())
    };
}

macro_rules! apply_effects {
    ( $e:expr, $effects:expr, $cmd:expr ) => {
        for effect in $effects.iter() {
            let effect_name = effect.0.as_str();
            match effect_name {
                "provides_healing" => $cmd.add_component(
                    $e,
                    ProvidesHealing {
                        amount: effect.1.parse::<i32>().unwrap(),
                    },
                ),
                "ranged" => $cmd.add_component($e, i32_component!(Ranged, effect)),
                "damage" => $cmd.add_component($e, i32_component!(Damage, effect)),
                "area_of_effect" => $cmd.add_component($e, i32_component!(AreaOfEffect, effect)),
                "confusion" => {
                    $cmd.add_component($e, Confusion);
                    $cmd.add_component($e, i32_component!(Duration, effect));
                }
                "magic_mapping" => $cmd.add_component($e, ProvidesDungeonMap),
                "town_portal" => $cmd.add_component($e, TownPortal),
                "food" => $cmd.add_component($e, ProvidesFood),
                "single_activation" => $cmd.add_component($e, SingleActivation),
                "particle_line" => $cmd.add_component($e, parse_particle_line(&effect.1)),
                "particle" => $cmd.add_component($e, parse_particle(&effect.1)),
                "remove_curse" => $cmd.add_component($e, ProvidesRemoveCurse),
                "identify" => $cmd.add_component($e, ProvidesIdentify),
                "provides_mana" => $cmd.add_component($e, i32_component!(ProvidesMana, effect)),
                "teach_spell" => $cmd.add_component($e, TeachSpell(effect.1.to_string())),
                "slow" => $cmd.add_component(
                    $e,
                    Slow {
                        initiative_penalty: effect.1.parse::<f32>().unwrap(),
                    },
                ),
                "damage_over_time" => $cmd.add_component(
                    $e,
                    DamageOverTime {
                        damage: effect.1.parse::<i32>().unwrap(),
                    },
                ),
                _ => log(format!(
                    "Warning: consumable effect {} not implemented.",
                    effect_name
                )),
            }
        }
    };
}

pub fn spawn_named_spell(
    raws: &RawMaster,
    key: &str,
    commands: &mut CommandBuffer,
) -> Option<Entity> {
    if let Some(idx) = raws.spell_index.get(key) {
        let spell_template = &raws.raws.spells[*idx];

        let entity = commands.push((
            SpellTemplate {
                mana_cost: spell_template.mana_cost,
            },
            Name(spell_template.name.clone()),
        ));
        apply_effects!(entity, spell_template.effects, commands);
    }
    None
}

pub fn spawn_all_spells(commands: &mut CommandBuffer) {
    let raws = &RAWS.lock().unwrap();
    for spell in raws.raws.spells.iter() {
        spawn_named_spell(raws, &spell.name, commands);
    }
}

pub fn find_spell_entity(ecs: &SubWorld, name: &str) -> Option<Entity> {
    <(Entity, &Name)>::query()
        .filter(component::<SpellTemplate>())
        .iter(ecs)
        .find_map(|(e, n)| if n.0 == name { Some(*e) } else { None })
}

pub fn spawn_named_item(
    raws: &RawMaster,
    key: &str,
    pos: SpawnType,
    dm: &MasterDungeonMap,
    commands: &mut CommandBuffer,
) -> Option<Entity> {
    if !raws.item_index.contains_key(key) {
        return None;
    }

    let item_template = &raws.raws.items[raws.item_index[key]];
    let scroll_names = dm.scroll_mappings.clone();
    let potion_names = dm.potion_mappings.clone();
    let wand_names = dm.wand_mappings.clone();
    let identified = dm.identified_items.clone();

    let item = Item {
        initiative_penalty: item_template.initiative_penalty.unwrap_or(0.0),
        weight_lbs: item_template.weight_lbs.unwrap_or(0.0),
        base_value: item_template.base_value.unwrap_or(0.0),
    };
    let entity = commands.push((item, Name(item_template.name.clone()), SerializeMe));

    // Spawn in the specified location
    set_position(&entity, pos, key, raws, commands);

    if let Some(renderable) = &item_template.renderable {
        commands.add_component(entity, get_renderable(&renderable));
    }

    if let Some(consumable) = &item_template.consumable {
        let max_charges = consumable.charges.unwrap_or(0);
        commands.add_component(
            entity,
            Consumable {
                max_charges,
                charges: i32::max(max_charges, 1),
            },
        );
        apply_effects!(entity, consumable.effects, commands);
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
            proc_chance: weapon.proc_chance,
            proc_target: weapon.proc_target.clone(),
        };
        match weapon.attribute.as_str() {
            "Quickness" => wpn.attribute = WeaponAttribute::Quickness,
            _ => wpn.attribute = WeaponAttribute::Might,
        }
        commands.add_component(entity, wpn);
        if let Some(proc_effects) = &weapon.proc_effects {
            apply_effects!(entity, proc_effects, commands);
        }
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

    if let Some(magic) = &item_template.magic {
        let class = match magic.class.as_str() {
            "rare" => MagicItemClass::Rare,
            "legendary" => MagicItemClass::Legendary,
            _ => MagicItemClass::Common,
        };
        commands.add_component(entity, MagicItem { class });

        if !identified.contains(&item_template.name) {
            match magic.naming.as_str() {
                "scroll" => commands.add_component(
                    entity,
                    ObfuscatedName(scroll_names[&item_template.name].clone()),
                ),
                "potion" => commands.add_component(
                    entity,
                    ObfuscatedName(potion_names[&item_template.name].clone()),
                ),
                "wand" => commands.add_component(
                    entity,
                    ObfuscatedName(wand_names[&item_template.name].clone()),
                ),
                _ => commands.add_component(entity, ObfuscatedName(magic.naming.clone())),
            }
        }

        if let Some(cursed) = magic.cursed {
            if cursed {
                commands.add_component(entity, CursedItem);
            }
        }
    }

    if let Some(ab) = &item_template.attributes {
        commands.add_component(
            entity,
            AttributeBonus {
                might: ab.might,
                fitness: ab.fitness,
                quickness: ab.quickness,
                intelligence: ab.intelligence,
            },
        )
    }

    Some(entity)
}

pub fn spawn_named_mob(
    raws: &RawMaster,
    key: &str,
    pos: SpawnType,
    dm: &MasterDungeonMap,
    commands: &mut CommandBuffer,
) -> Option<Entity> {
    if !raws.mob_index.contains_key(key) {
        return None;
    }
    let mob_template = &raws.raws.mobs[raws.mob_index[key]];

    let entity = commands.push(((), Name(mob_template.name.clone()), SerializeMe));
    set_position(&entity, pos, key, raws, commands);

    match mob_template.movement.as_ref() {
        "random" => commands.add_component(entity, MoveMode(Movement::Random)),
        "random_waypoint" => {
            commands.add_component(entity, MoveMode(Movement::RandomWaypoint { path: None }))
        }
        _ => commands.add_component(entity, MoveMode(Movement::Static)),
    }

    if let Some(quips) = &mob_template.quips {
        commands.add_component(entity, Quips(quips.clone()));
    }

    if let Some(renderable) = &mob_template.renderable {
        commands.add_component(entity, get_renderable(renderable));
    }

    if let Some(categories) = &mob_template.vendor {
        commands.add_component(
            entity,
            Vendor {
                categories: categories.clone(),
            },
        )
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

    commands.add_component(entity, Initiative { current: 2 });

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
    let mob_gold = mob_template
        .gold
        .as_ref()
        .map(|gold| {
            let mut rng = RandomNumberGenerator::new();
            rng.roll_str(gold).map_or(0.0, |v| v as f32)
        })
        .unwrap_or(0.0);
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
            total_weight: 0.0,
            total_initiative_penalty: 0.0,
            gold: mob_gold,
            god_mode: false,
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
            spawn_named_entity(raws, tag, SpawnType::Equipped { by: entity }, dm, commands);
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

    if let Some(light) = &mob_template.light {
        commands.add_component(
            entity,
            LightSource {
                range: light.range,
                color: RGB::from_hex(&light.color).expect("Bad color"),
            },
        )
    }

    if let Some(faction) = &mob_template.faction {
        commands.add_component(
            entity,
            Faction {
                name: faction.clone(),
            },
        );
    } else {
        commands.add_component(
            entity,
            Faction {
                name: "Mindless".to_string(),
            },
        )
    }
    commands.add_component(entity, EquipmentChanged);

    if let Some(ability_list) = &mob_template.abilities {
        let mut a = SpecialAbilities {
            abilities: Vec::new(),
        };
        for ability in ability_list.iter() {
            a.abilities.push(SpecialAbility {
                spell: ability.spell.clone(),
                chance: ability.chance,
                range: ability.range,
                min_range: ability.min_range,
            });
        }
        commands.add_component(entity, a);
    }

    Some(entity)
}

pub fn spawn_named_prop(
    raws: &RawMaster,
    key: &str,
    pos: SpawnType,
    commands: &mut CommandBuffer,
) -> Option<Entity> {
    if !raws.prop_index.contains_key(key) {
        return None;
    }
    let template = &raws.raws.props[raws.prop_index[key]];

    let entity = commands.push(((), Name(template.name.clone()), SerializeMe));
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
        apply_effects!(entity, entry_trigger.effects, commands);
    }
    if let Some(light) = &template.light {
        commands.add_component(
            entity,
            LightSource {
                color: RGB::from_hex(&light.color).unwrap(),
                range: light.range,
            },
        );
        commands.add_component(
            entity,
            FieldOfView {
                visible_tiles: HashSet::new(),
                radius: light.range,
                is_dirty: true,
            },
        );
    }

    Some(entity)
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    key: &str,
    pos: SpawnType,
    dm: &MasterDungeonMap,
    commands: &mut CommandBuffer,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        spawn_named_item(raws, key, pos, dm, commands)
    } else if raws.mob_index.contains_key(key) {
        spawn_named_mob(raws, key, pos, dm, commands)
    } else if raws.prop_index.contains_key(key) {
        spawn_named_prop(raws, key, pos, commands)
    } else {
        None
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

pub fn faction_reaction(my_faction: &str, their_faction: &str, raws: &RawMaster) -> Reaction {
    if raws.faction_index.contains_key(my_faction) {
        let mf = &raws.faction_index[my_faction];
        if mf.contains_key(their_faction) {
            return mf[their_faction];
        } else if mf.contains_key("Default") {
            return mf["Default"];
        } else {
            return Reaction::Ignore;
        }
    }
    Reaction::Ignore
}

pub fn get_vendor_items(categories: &[String], raws: &RawMaster) -> Vec<(String, f32)> {
    let mut result: Vec<(String, f32)> = Vec::new();

    for item in raws.raws.items.iter() {
        if let Some(cat) = &item.vendor_category {
            if categories.contains(cat) && item.base_value.is_some() {
                result.push((item.name.clone(), item.base_value.unwrap()));
            }
        }
    }

    result
}

pub fn get_item_color(ecs: &SubWorld, item: Entity, dm: &MasterDungeonMap) -> ColorPair {
    let entry = ecs.entry_ref(item).unwrap();
    if entry.get_component::<CursedItem>().is_ok() {
        if let Ok(name) = entry.get_component::<Name>() {
            if dm.identified_items.contains(&name.0) {
                return ColorPair::new(RED, BLACK);
            }
        }
    }

    let fg = if let Ok(magic) = entry.get_component::<MagicItem>() {
        match magic.class {
            MagicItemClass::Common => RGB::from_f32(0.5, 1.0, 0.5),
            MagicItemClass::Rare => RGB::from_f32(0.0, 1.0, 1.0),
            MagicItemClass::Legendary => RGB::from_f32(0.71, 0.15, 0.93),
        }
    } else {
        RGB::named(WHITE)
    };
    ColorPair::new(fg, BLACK)
}

pub fn get_scroll_tags() -> Vec<String> {
    let raws = &super::RAWS.lock().unwrap();
    let mut result = Vec::new();

    for item in raws.raws.items.iter() {
        if let Some(magic) = &item.magic {
            if &magic.naming == "scroll" {
                result.push(item.name.clone());
            }
        }
    }

    result
}

pub fn get_potion_tags() -> Vec<String> {
    let raws = &super::RAWS.lock().unwrap();
    let mut result = Vec::new();

    for item in raws.raws.items.iter() {
        if let Some(magic) = &item.magic {
            if &magic.naming == "potion" {
                result.push(item.name.clone());
            }
        }
    }

    result
}

pub fn get_wand_tags() -> Vec<String> {
    let raws = &super::RAWS.lock().unwrap();
    let mut result = Vec::new();

    for item in raws.raws.items.iter() {
        if let Some(magic) = &item.magic {
            if &magic.naming == "wand" {
                result.push(item.name.clone());
            }
        }
    }

    result
}

pub fn get_item_display_name(ecs: &SubWorld, item: Entity, dm: &MasterDungeonMap) -> String {
    if let Ok(entry) = ecs.entry_ref(item) {
        if let Ok(name) = entry.get_component::<Name>() {
            if entry.get_component::<MagicItem>().is_ok() {
                if dm.identified_items.contains(&name.0) {
                    if let Ok(c) = entry.get_component::<Consumable>() {
                        if c.max_charges > 0 {
                            format!("{} ({})", &name.0, c.charges).to_string()
                        } else {
                            name.0.clone()
                        }
                    } else {
                        name.0.clone()
                    }
                } else if let Ok(obfuscated) = entry.get_component::<ObfuscatedName>() {
                    obfuscated.0.clone()
                } else {
                    "Unidentified magic item".to_string()
                }
            } else {
                name.0.clone()
            }
        } else {
            "Nameless item (bug)".to_string()
        }
    } else {
        "Unknown item (bug)".to_string()
    }
}

pub fn is_tag_magic(tag: &str) -> bool {
    let raws = &RAWS.lock().unwrap();
    if let Some(idx) = raws.item_index.get(tag) {
        let item_template = &raws.raws.items[*idx];
        item_template.magic.is_some()
    } else {
        false
    }
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
