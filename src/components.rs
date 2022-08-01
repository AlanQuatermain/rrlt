use std::collections::{HashMap, HashSet};

use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Render {
    pub color: ColorPair,
    pub glyph: FontCharType,
    pub render_order: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub map_level: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Name(pub String);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToAttack {
    pub attacker: Entity,
    pub victim: Entity,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub initiative_penalty: f32,
    pub weight_lbs: f32,
    pub base_value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AmuletOfYala;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct FieldOfView {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub visible_tiles: HashSet<Point>,
    pub radius: i32,
    pub is_dirty: bool,
}

impl FieldOfView {
    pub fn new(radius: i32) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            radius,
            is_dirty: true,
        }
    }

    pub fn clone_dirty(&self) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            radius: self.radius,
            is_dirty: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvidesHealing {
    pub amount: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvidesDungeonMap;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Carried(pub Entity);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Damage(pub i32);

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum WeaponAttribute {
    Might,
    Quickness,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeleeWeapon {
    pub attribute: WeaponAttribute,
    pub damage_die: String,
    pub hit_bonus: i32,
}

impl Default for MeleeWeapon {
    fn default() -> Self {
        Self {
            attribute: WeaponAttribute::Might,
            damage_die: "1d4".to_string(),
            hit_bonus: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wearable {
    pub armor_class: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlocksTile;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToCollect {
    pub who: Entity,
    pub what: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToDrop {
    pub who: Entity,
    pub what: Entity,
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
    Melee,
    Shield,
    Head,
    Torso,
    Legs,
    Feet,
    Hands,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParticleLifetime(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InflictDamage {
    pub target: Entity,
    pub user_entity: Entity,
    pub damage: i32,
    pub item_entity: Option<Entity>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvidesFood;

#[derive(Copy, Clone, Debug)]
pub struct Consumed;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hidden;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntryTrigger;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EntityMoved;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SingleActivation;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlocksVisibility {}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Door {
    pub open: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlwaysVisible;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quips(pub Vec<String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub base: i32,
    pub modifiers: i32,
    pub bonus: i32,
}

impl Default for Attribute {
    fn default() -> Self {
        Self {
            base: 11,
            modifiers: 0,
            bonus: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub might: Attribute,
    pub fitness: Attribute,
    pub quickness: Attribute,
    pub intelligence: Attribute,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            might: Default::default(),
            fitness: Default::default(),
            quickness: Default::default(),
            intelligence: Default::default(),
        }
    }
}

impl Attributes {
    pub fn max_weight(&self) -> i32 {
        (self.might.base + self.might.modifiers) * 15
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Skill {
    Melee,
    Defense,
    Magic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills(pub HashMap<Skill, i32>);

impl Default for Skills {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(Skill::Melee, 1);
        map.insert(Skill::Defense, 1);
        map.insert(Skill::Magic, 1);
        Self(map)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub max: i32,
    pub current: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pools {
    pub hit_points: Pool,
    pub mana: Pool,
    pub xp: i32,
    pub level: i32,
    pub total_weight: f32,
    pub total_initiative_penalty: f32,
    pub gold: f32,
    pub god_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalAttack {
    pub name: String,
    pub damage_die: String,
    pub hit_bonus: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalAttackDefense {
    pub armor_class: i32,
    pub attacks: Vec<NaturalAttack>,
}

impl Default for NaturalAttackDefense {
    fn default() -> Self {
        Self {
            armor_class: 10,
            attacks: Vec::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootTable(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OtherLevelPosition {
    pub position: Point,
    pub depth: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LightSource {
    pub color: RGB,
    pub range: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Initiative {
    pub current: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyTurn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantsToApproach {
    pub idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WantsToFlee {
    pub indices: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Movement {
    Static,
    Random,
    RandomWaypoint { path: Option<Vec<usize>> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveMode(pub Movement);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chasing {
    pub target: Entity,
}

#[derive(Debug, Copy, Clone)]
pub struct EquipmentChanged;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TownPortal;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TeleportTo {
    pub position: Point,
    pub depth: i32,
    pub player_only: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct WantsToMove {
    pub destination: Point,
}

#[derive(Debug, Copy, Clone)]
pub struct ApplyTeleport {
    pub destination: Point,
    pub depth: i32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MagicItemClass {
    Common,
    Rare,
    Legendary,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MagicItem {
    pub class: MagicItemClass,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObfuscatedName(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentifiedItem(pub String);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UseItem {
    pub user: Entity,
    pub target: Option<Point>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpawnParticleLine {
    pub glyph: FontCharType,
    pub color: RGB,
    pub lifetime_ms: f32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpawnParticleBurst {
    pub glyph: FontCharType,
    pub color: RGB,
    pub lifetime_ms: f32,
}
