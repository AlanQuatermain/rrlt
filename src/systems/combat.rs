use super::*;
use crate::prelude::*;

#[system(for_each)]
#[read_component(WantsToAttack)]
#[read_component(Player)]
#[read_component(Pools)]
#[read_component(Damage)]
#[read_component(Armor)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(Equipped)]
#[read_component(Point)]
#[read_component(Attributes)]
#[read_component(Skills)]
#[read_component(HungerClock)]
pub fn combat(
    message: &Entity,
    wants_attack: &WantsToAttack,
    ecs: &mut SubWorld,
    #[resource] log: &mut Gamelog,
    #[resource] rng: &mut RandomNumberGenerator,
    #[resource] particle_builder: &mut ParticleBuilder,
    commands: &mut CommandBuffer,
) {
    let (attacker, victim) = (wants_attack.attacker, wants_attack.victim);
    let attacker_name = name_for(&attacker, ecs).0;
    let victim_name = name_for(&victim, ecs).0;

    let attacker_entry = ecs.entry_ref(attacker).unwrap();
    let victim_entry = ecs.entry_ref(victim).unwrap();

    let attacker_stats = attacker_entry.get_component::<Pools>().unwrap();
    let attacker_attrs = attacker_entry.get_component::<Attributes>().unwrap();
    let attacker_skills = attacker_entry.get_component::<Skills>().unwrap();

    let victim_stats = victim_entry.get_component::<Pools>().unwrap();
    let victim_attrs = victim_entry.get_component::<Attributes>().unwrap();
    let victim_skills = victim_entry.get_component::<Skills>().unwrap();

    // Are the attacker and defender alive? Only attack if they are.
    if attacker_stats.hit_points.current <= 0 || victim_stats.hit_points.current <= 0 {
        return;
    }

    let natural_roll = rng.roll_dice(1, 20);
    let attr_hit_bonus = attacker_attrs.might.bonus;
    let skill_hit_bonus = skill_bonus(Skill::Melee, attacker_skills);
    let weapon_hit_bonus = 0; // TODO: once weapons support this
    let mut status_hit_bonus = 0;
    if let Ok(hc) = attacker_entry.get_component::<HungerClock>() {
        if hc.state == HungerState::WellFed {
            status_hit_bonus += 1;
        }
    }
    let modified_hit_roll =
        natural_roll + attr_hit_bonus + skill_hit_bonus + weapon_hit_bonus + status_hit_bonus;

    let base_armor_class = 10;
    let armor_quickness_bonus = victim_attrs.quickness.bonus;
    let armor_skill_bonus = skill_bonus(Skill::Defense, victim_skills);
    let armor_item_bonus = 0; // TODO: once armor supports this
    let armor_class =
        base_armor_class + armor_quickness_bonus + armor_skill_bonus + armor_item_bonus;

    if natural_roll != 1 && (natural_roll == 20 || modified_hit_roll > armor_class) {
        // Target hit! Until we support weapons, we're doing with 1d4
        let base_damage = rng.roll_dice(1, 4);
        let attr_damage_bonus = attacker_attrs.might.bonus;
        let skill_damage_bonus = skill_bonus(Skill::Melee, attacker_skills);
        let weapon_damage_bonus = 0;

        let damage = i32::max(
            0,
            base_damage + attr_damage_bonus + skill_damage_bonus + weapon_damage_bonus,
        );
        commands.push((
            (),
            InflictDamage {
                target: victim,
                user_entity: attacker,
                damage,
                item_entity: None,
            },
        ));
    } else if natural_roll == 1 {
        // Natural 1 miss
        log.entries.push(format!(
            "{} considers attacking {}, but misjudges the timing.",
            attacker_name, victim_name
        ));
        particle_builder.request(
            *(victim_entry.get_component::<Point>().unwrap()),
            ColorPair::new(BLUE, BLACK),
            to_cp437('‼'),
            200.0,
        );
    } else {
        // Miss
        log.entries.push(format!(
            "{} attacks {}, but can't connect.",
            attacker_name, victim_name
        ));
        particle_builder.request(
            *(victim_entry.get_component::<Point>().unwrap()),
            ColorPair::new(CYAN, BLACK),
            to_cp437('‼'),
            200.0,
        );
    }

    commands.remove(*message);
}
