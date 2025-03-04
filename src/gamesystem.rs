use crate::prelude::*;

pub fn attr_bonus(value: i32) -> i32 {
    (value - 10) / 2
}

pub fn player_hp_per_level(fitness: i32) -> i32 {
    15 + attr_bonus(fitness)
}

pub fn player_hp_at_level(fitness: i32, level: i32) -> i32 {
    15 + (player_hp_per_level(fitness) * level)
}

pub fn npc_hp(fitness: i32, level: i32) -> i32 {
    let mut total = 1;
    for _ in 0..level {
        total += i32::max(1, 8 + attr_bonus(fitness));
    }
    total
}

pub fn mana_per_level(intelligence: i32) -> i32 {
    i32::max(1, 4 + attr_bonus(intelligence))
}

pub fn mana_at_level(intelligence: i32, level: i32) -> i32 {
    mana_per_level(intelligence) * level
}

pub fn skill_bonus(skill: Skill, skills: &Skills) -> i32 {
    if skills.0.contains_key(&skill) {
        skills.0[&skill]
    } else {
        -4
    }
}
