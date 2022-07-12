use crate::prelude::*;
use super::*;

#[system(for_each)]
#[read_component(WantsToAttack)]
#[read_component(Player)]
#[write_component(Health)]
#[read_component(Damage)]
#[read_component(Armor)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(Equipped)]
#[read_component(Point)]
#[read_component(HungerClock)]
pub fn combat(
    message: &Entity,
    wants_attack: &WantsToAttack,
    ecs: &mut SubWorld,
    #[resource] log: &mut Gamelog,
    commands: &mut CommandBuffer,
) {
    let (attacker, victim) = (wants_attack.attacker, wants_attack.victim);
    let is_player = ecs.entry_ref(victim)
        .unwrap().get_component::<Player>().is_ok();
    let attacker_name = name_for(&attacker, ecs).0;
    let victim_name = name_for(&victim, ecs).0;

    let base_damage = if let Ok(v) = ecs.entry_ref(attacker) {
        if let Ok(dmg) = v.get_component::<Damage>() {
            dmg.0
        }
        else {
            0
        }
    }
    else {
        0
    };

    let weapon_damage: i32 = <(&Equipped, &Damage)>::query()
        .filter(component::<Weapon>())
        .iter(ecs)
        .filter(|(equipped, _)| equipped.owner == attacker)
        .map(|(_, dmg)| dmg.0)
        .sum();
    let well_fed_bonus = <(&HungerClock, Entity)>::query()
        .iter(ecs)
        .filter(|(_, entity)| **entity == attacker)
        .map(|(clock, _)| {
            if clock.state == HungerState::WellFed { 1 } else { 0 }
        })
        .nth(0)
        .unwrap_or(0);
    let defense = if let Ok(v) = ecs.entry_ref(victim) {
        let base_armor = if let Ok(armor) = v.get_component::<Armor>() {
            armor.0
        } else { 0 };
        let equipped_armor: i32 = <(&Equipped, &Armor)>::query()
            .iter(ecs)
            .filter(|(equipped, _)| equipped.owner == victim)
            .map(|(_, armor)| armor.0)
            .sum();
        base_armor + equipped_armor
    } else { 0 };
    let final_damage = base_damage + weapon_damage + well_fed_bonus - defense;

    if final_damage <= 0 {
        log.entries.push(format!("{} is unable to hurt {}", attacker_name, victim_name));
    }
    else {
        commands.push(((), InflictDamage {
            target: victim,
            user_entity: attacker,
            damage: final_damage,
            item_entity: None
        }));
    }
    commands.remove(*message);
}