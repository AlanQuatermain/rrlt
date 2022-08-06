use crate::prelude::*;
use std::{
    collections::{HashSet, VecDeque},
    sync::Mutex,
};

mod damage;
mod hunger;
mod identify;
mod movement;
mod particles;
mod targeting;
mod triggers;

pub use targeting::*;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType {
    Damage {
        amount: i32,
    },
    Bloodstain,
    Particle {
        glyph: FontCharType,
        color: ColorPair,
        lifespan: f32,
    },
    ParticleProjectile {
        glyph: FontCharType,
        color: ColorPair,
        lifespan: f32,
        speed: f32,
        path: Vec<Point>,
    },
    EntityDeath,
    Identify,
    ItemUse {
        item: Entity,
    },
    CastSpell {
        spell: Entity,
    },
    WellFed,
    Healing {
        amount: i32,
    },
    Mana {
        amount: i32,
    },
    Confusion {
        turns: i32,
    },
    TriggerFire {
        trigger: Entity,
    },
    TeleportTo {
        pos: Point,
        depth: i32,
        player_only: bool,
    },
    AttributeEffect {
        bonus: AttributeBonus,
        name: String,
        duration: i32,
    },
    LearnSpell {
        name: String,
        spell: Entity,
    },
    Slow {
        initiative_penalty: f32,
    },
    DamageOverTime {
        damage: i32,
    },
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum Targets {
    Single { target: Entity },
    Area { targets: Vec<Entity> },
    Tile { tile_idx: usize },
    Tiles { tiles: Vec<usize> },
}

pub struct EffectSpawner {
    pub creator: Option<Entity>,
    pub effect_type: EffectType,
    pub targets: Targets,
}

pub fn add_effect(creator: Option<Entity>, effect_type: EffectType, targets: Targets) {
    EFFECT_QUEUE.lock().unwrap().push_back(EffectSpawner {
        creator,
        effect_type,
        targets,
    });
}

pub fn run_effects_queue(
    ecs: &mut SubWorld,
    map: &mut Map,
    particle_builder: &mut ParticleBuilder,
    gamelog: &mut Gamelog,
    turn_state: &mut TurnState,
    dm: &mut MasterDungeonMap,
    commands: &mut CommandBuffer,
) {
    loop {
        let effect: Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            target_applicator(
                ecs,
                &effect,
                map,
                particle_builder,
                gamelog,
                turn_state,
                dm,
                commands,
            );
        } else {
            break;
        }
    }
}

fn target_applicator(
    ecs: &mut SubWorld,
    effect: &EffectSpawner,
    map: &mut Map,
    particle_builder: &mut ParticleBuilder,
    gamelog: &mut Gamelog,
    turn_state: &mut TurnState,
    dm: &mut MasterDungeonMap,
    commands: &mut CommandBuffer,
) {
    if let EffectType::ItemUse { item } = effect.effect_type {
        triggers::item_trigger(
            effect.creator,
            item,
            &effect.targets,
            ecs,
            gamelog,
            particle_builder,
            turn_state,
            map,
            commands,
        );
        return;
    }

    if let EffectType::CastSpell { spell } = effect.effect_type {
        triggers::spell_trigger(
            effect.creator,
            spell,
            &effect.targets,
            ecs,
            gamelog,
            particle_builder,
            turn_state,
            map,
            commands,
        );
        return;
    }

    if let EffectType::TriggerFire { trigger } = effect.effect_type {
        triggers::trigger(
            effect.creator,
            trigger,
            &effect.targets,
            ecs,
            gamelog,
            particle_builder,
            turn_state,
            map,
            commands,
        );
        return;
    }

    match &effect.targets {
        Targets::Tile { tile_idx } => affect_tile(
            ecs,
            effect,
            *tile_idx,
            map,
            particle_builder,
            gamelog,
            turn_state,
            dm,
            commands,
        ),
        Targets::Tiles { tiles } => tiles.iter().for_each(|tile_idx| {
            affect_tile(
                ecs,
                effect,
                *tile_idx,
                map,
                particle_builder,
                gamelog,
                turn_state,
                dm,
                commands,
            )
        }),
        Targets::Single { target } => affect_entity(
            ecs,
            effect,
            *target,
            map,
            particle_builder,
            gamelog,
            turn_state,
            dm,
            commands,
        ),
        Targets::Area { targets } => targets.iter().for_each(|target| {
            affect_entity(
                ecs,
                effect,
                *target,
                map,
                particle_builder,
                gamelog,
                turn_state,
                dm,
                commands,
            )
        }),
    }
}

fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage { .. } => true,
        EffectType::WellFed => true,
        EffectType::Healing { .. } => true,
        EffectType::Confusion { .. } => true,
        EffectType::TeleportTo { .. } => true,
        EffectType::AttributeEffect { .. } => true,
        EffectType::Mana { .. } => true,
        EffectType::LearnSpell { .. } => true,
        EffectType::Slow { .. } => true,
        EffectType::DamageOverTime { .. } => true,
        _ => false,
    }
}

fn affect_tile(
    ecs: &mut SubWorld,
    effect: &EffectSpawner,
    tile_idx: usize,
    map: &mut Map,
    particle_builder: &mut ParticleBuilder,
    gamelog: &mut Gamelog,
    turn_state: &mut TurnState,
    dm: &mut MasterDungeonMap,
    commands: &mut CommandBuffer,
) {
    if tile_effect_hits_entities(&effect.effect_type) {
        let pos = map.index_to_point2d(tile_idx);
        let content: HashSet<Entity> = <(Entity, &Point, Option<&TileSize>)>::query()
            .iter(ecs)
            .filter(|(_, p, s)| {
                if let Some(size) = s {
                    Rect::with_size(p.x, p.y, size.x, size.y).point_in_rect(pos)
                } else {
                    **p == pos
                }
            })
            .map(|(e, _, _)| *e)
            .collect();
        content.iter().for_each(|entity| {
            affect_entity(
                ecs,
                effect,
                *entity,
                map,
                particle_builder,
                gamelog,
                turn_state,
                dm,
                commands,
            )
        });
    }

    match &effect.effect_type {
        EffectType::Bloodstain => damage::bloodstain(map, vec![tile_idx]),
        EffectType::Particle { .. } => {
            particles::particle_to_tile(ecs, vec![tile_idx], effect, map, particle_builder)
        }
        EffectType::ParticleProjectile { .. } => {
            particles::projectile(ecs, tile_idx, effect, map, particle_builder)
        }
        _ => {}
    }
}

fn affect_entity(
    ecs: &mut SubWorld,
    effect: &EffectSpawner,
    target: Entity,
    map: &mut Map,
    particle_builder: &mut ParticleBuilder,
    gamelog: &mut Gamelog,
    _turn_state: &mut TurnState,
    dm: &mut MasterDungeonMap,
    commands: &mut CommandBuffer,
) {
    match &effect.effect_type {
        EffectType::Damage { .. } => damage::inflict_damage(ecs, effect, map, gamelog, target),
        EffectType::EntityDeath => damage::death(ecs, effect, target, map, gamelog),
        EffectType::Bloodstain => {
            if let Some(pos) = entity_position(ecs, target, map) {
                damage::bloodstain(map, pos)
            }
        }
        EffectType::Particle { .. } => {
            if let Some(pos) = entity_position(ecs, target, map) {
                particles::particle_to_tile(ecs, pos, effect, map, particle_builder);
            }
        }
        EffectType::Identify => identify::identify_entity(ecs, effect, target, dm, commands),
        EffectType::WellFed => hunger::well_fed(ecs, target),
        EffectType::Healing { .. } => damage::heal_damage(ecs, effect, target),
        EffectType::Mana { .. } => damage::restore_mana(ecs, effect, target),
        EffectType::Confusion { .. } => damage::add_confusion(ecs, effect, target, commands),
        EffectType::TeleportTo { .. } => movement::apply_teleport(ecs, effect, target, commands),
        EffectType::AttributeEffect { .. } => {
            damage::attribute_effect(ecs, effect, target, commands)
        }
        EffectType::LearnSpell { name, spell } => {
            triggers::learn_spell(ecs, effect, name.to_string(), *spell, commands)
        }
        EffectType::Slow { .. } => damage::slow(ecs, effect, target, commands),
        EffectType::DamageOverTime { .. } => {
            damage::damage_over_time(ecs, effect, target, commands)
        }
        _ => {}
    }
}
