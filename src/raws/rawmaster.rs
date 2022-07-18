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
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }

        self.mob_index = HashMap::new();
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            self.mob_index.insert(mob.name.clone(), i);
        }

        self.prop_index = HashMap::new();
        for (i, prop) in self.raws.props.iter().enumerate() {
            self.prop_index.insert(prop.name.clone(), i);
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

    let entity = commands.push((
        Enemy,
        ChasingPlayer,
        Name(mob_template.name.clone()),
        get_position(pos),
    ));

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
    if let Some(hidden) = &template.hidden {
        commands.add_component(entity, Hidden);
    }
    if let Some(blocks_tile) = &template.blocks_tile {
        commands.add_component(entity, BlocksTile);
    }
    if let Some(blocks_visibility) = &template.blocks_visibility {
        commands.add_component(entity, BlocksVisibility {});
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
