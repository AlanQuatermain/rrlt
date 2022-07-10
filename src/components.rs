use std::collections::HashSet;

use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Render {
    pub color: ColorPair,
    pub glyph: FontCharType
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub map_level: u32
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Enemy;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WantsToMove {
    pub entity: Entity,
    pub destination: Point
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Health {
    pub current: i32,
    pub max: i32
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Name(pub String);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToAttack {
    pub attacker: Entity,
    pub victim: Entity
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChasingPlayer;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AmuletOfYala;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct FieldOfView {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub visible_tiles: HashSet<Point>,
    pub radius: i32,
    pub is_dirty: bool
}

impl FieldOfView {
    pub fn new(radius: i32) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            radius,
            is_dirty: true
        }
    }

    pub fn clone_dirty(&self) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            radius: self.radius,
            is_dirty: true
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvidesHealing {
    pub amount: i32
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvidesDungeonMap;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Carried(pub Entity);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivateItem {
    pub used_by: Entity,
    pub item: Entity,
    pub target: Option<Point>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Damage(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Weapon;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlocksTile;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Armor(pub i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToCollect {
    pub who: Entity,
    pub what: Entity
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToDrop {
    pub who: Entity,
    pub what: Entity
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Consumable;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ranged(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AreaOfEffect(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Confusion(pub i32);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SerializeMe;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee, Shield
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot
}