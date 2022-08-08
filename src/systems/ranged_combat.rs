use super::*;
use crate::prelude::*;

#[system(for_each)]
#[read_component(WantsToShoot)]
#[read_component(Player)]
#[read_component(Pools)]
#[read_component(Damage)]
#[read_component(Wearable)]
#[read_component(Weapon)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(Equipped)]
#[read_component(Point)]
#[read_component(Attributes)]
#[read_component(Skills)]
#[read_component(HungerClock)]
#[read_component(NaturalAttackDefense)]
pub fn ranged_combat(
    attacker: &Entity,
    wants_attack: &WantsToShoot,
    player: Option<&Player>,
    attacker_pos: &Point,
    attacker_stats: &Pools,
    attacker_attrs: &Attributes,
    attacker_skills: &Skills,
    attacker_name: Option<&Name>,
    attacker_natural: Option<&NaturalAttackDefense>,
    hunger_clock: Option<&HungerClock>,
    ecs: &mut SubWorld,
    #[resource] rng: &mut RandomNumberGenerator,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
) {
    let victim = wants_attack.target;
    let attacker_name = if player.is_some() {
        "You".to_string()
    } else {
        attacker_name
            .map(|n| n.0.clone())
            .unwrap_or("Someone".to_string())
    };
    let victim_name = name_for(&victim, ecs).0;

    let victim_entry = ecs.entry_ref(victim).unwrap();

    let victim_stats = victim_entry.get_component::<Pools>().unwrap();
    let victim_attrs = victim_entry.get_component::<Attributes>().unwrap();
    let victim_skills = victim_entry.get_component::<Skills>().unwrap();

    // Are the attacker and defender alive? Only attack if they are.
    if attacker_stats.hit_points.current <= 0 || victim_stats.hit_points.current <= 0 {
        return;
    }

    let victim_pos = victim_entry.get_component::<Point>().unwrap();
    add_effect(
        None,
        EffectType::ParticleProjectile {
            glyph: to_cp437('*'),
            color: ColorPair::new(CYAN, BLACK),
            lifespan: 300.0,
            speed: 50.0,
            path: line2d_bresenham(*attacker_pos, *victim_pos),
        },
        Targets::Tile {
            tile_idx: map.point2d_to_index(*attacker_pos),
        },
    );

    // Find the attacker's weapon.
    let (mut weapon_info, weapon_entity) = <(&Weapon, &Equipped, Entity)>::query()
        .iter(ecs)
        .filter(|(_, e, _)| e.owner == *attacker)
        .find_map(|(wpn, _, e)| Some((wpn.clone(), Some(*e))))
        .unwrap_or_default();
    if let Some(nat) = attacker_natural {
        if !nat.attacks.is_empty() {
            let attack = rng.random_slice_entry(nat.attacks.as_slice()).unwrap();
            weapon_info.hit_bonus = attack.hit_bonus;
            weapon_info.damage_die = attack.damage_die.clone();
        }
    }

    let natural_roll = rng.roll_dice(1, 20);
    let attr_hit_bonus = match weapon_info.attribute {
        WeaponAttribute::Might => attacker_attrs.might.bonus,
        WeaponAttribute::Quickness => attacker_attrs.quickness.bonus,
    };
    let skill_hit_bonus = skill_bonus(Skill::Melee, attacker_skills);
    let weapon_hit_bonus = weapon_info.hit_bonus;
    let mut status_hit_bonus = 0;
    if let Some(hc) = hunger_clock {
        if hc.state == HungerState::WellFed {
            status_hit_bonus += 1;
        }
    }
    let modified_hit_roll =
        natural_roll + attr_hit_bonus + skill_hit_bonus + weapon_hit_bonus + status_hit_bonus;
    // println!(
    //     "Natural hit roll: {}, modified: {}",
    //     natural_roll, modified_hit_roll
    // );

    let armor_item_bonus_f: f32 = <(&Wearable, &Equipped)>::query()
        .iter(ecs)
        .filter(|(_, e)| e.owner == victim)
        .map(|(item, _)| item.armor_class)
        .sum();

    let base_armor_class = victim_entry
        .get_component::<NaturalAttackDefense>()
        .map(|n| n.armor_class)
        .unwrap_or(10);
    let armor_quickness_bonus = victim_attrs.quickness.bonus;
    let armor_skill_bonus = skill_bonus(Skill::Defense, victim_skills);
    let armor_item_bonus = armor_item_bonus_f as i32;
    let armor_class =
        base_armor_class + armor_quickness_bonus + armor_skill_bonus + armor_item_bonus;
    // println!("Armor class: {}", armor_class);

    if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
        let base_damage = rng
            .roll_str(&weapon_info.damage_die)
            .expect("Failed to parse die roll");
        let attr_damage_bonus = attacker_attrs.might.bonus;
        let skill_damage_bonus = skill_bonus(Skill::Melee, attacker_skills);

        let amount = i32::max(0, base_damage + attr_damage_bonus + skill_damage_bonus);
        // println!(
        //     "Damage: {} + {}attr + {}skill + {}weapon = {}",
        //     base_damage, attr_damage_bonus, skill_damage_bonus, &weapon_info.damage_die, amount,
        // );
        add_effect(
            Some(*attacker),
            EffectType::Damage { amount },
            Targets::Single { target: victim },
        );

        if let Some(chance) = weapon_info.proc_chance {
            if rng.roll_dice(1, 100) <= (chance * 100.0) as i32 && weapon_entity.is_some() {
                let effect_target = if weapon_info.proc_target.unwrap() == "Self" {
                    Targets::Single { target: *attacker }
                } else {
                    Targets::Single { target: victim }
                };
                add_effect(
                    Some(*attacker),
                    EffectType::ItemUse {
                        item: weapon_entity.unwrap(),
                    },
                    effect_target,
                );
            }
        }
    } else if natural_roll == 1 {
        // Natural 1 miss
        crate::gamelog::Logger::new()
            .npc_name(&attacker_name)
            .append("considers attacking")
            .npc_name(&victim_name)
            .append("but misjudges the timing!")
            .log();
        add_effect(
            None,
            EffectType::Particle {
                glyph: to_cp437('‼'),
                color: ColorPair::new(BLUE, BLACK),
                lifespan: 200.0,
            },
            Targets::Single { target: victim },
        );
    } else {
        // Miss
        crate::gamelog::Logger::new()
            .npc_name(&attacker_name)
            .append("attacks")
            .npc_name(&victim_name)
            .append("but can't connect.")
            .log();
        add_effect(
            None,
            EffectType::Particle {
                glyph: to_cp437('‼'),
                color: ColorPair::new(CYAN, BLACK),
                lifespan: 200.0,
            },
            Targets::Single { target: victim },
        );
    }

    commands.remove_component::<WantsToShoot>(*attacker);
}
