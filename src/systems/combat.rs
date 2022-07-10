use crate::prelude::*;
use super::*;

#[system]
#[read_component(WantsToAttack)]
#[read_component(Player)]
#[write_component(Health)]
#[read_component(Damage)]
#[read_component(Armor)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(Equipped)]
pub fn combat(
    ecs: &mut SubWorld,
    #[resource] log: &mut Gamelog,
    #[resource] turn_state: &mut TurnState,
    commands: &mut CommandBuffer,
) {
    let mut attackers = <(Entity, &WantsToAttack)>::query();
    let victims: Vec<(Entity, Entity, Entity)> = attackers
        .iter(ecs)
        .map(|(entity, attack)| (*entity, attack.attacker, attack.victim))
        .collect();

    victims.iter().for_each(|(message, attacker, victim)| {
        let is_player = ecs.entry_ref(*victim)
            .unwrap().get_component::<Player>().is_ok();
        let attacker_name = name_for(attacker, ecs).0;
        let victim_name = name_for(victim, ecs).0;

        let base_damage = if let Ok(v) = ecs.entry_ref(*attacker) {
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
            .filter(|(equipped, _)| equipped.owner == *attacker)
            .map(|(_, dmg)| dmg.0)
            .sum();
        let defense = if let Ok(v) = ecs.entry_ref(*victim) {
            let base_armor = if let Ok(armor) = v.get_component::<Armor>() {
                armor.0
            } else { 0 };
            let equipped_armor: i32 = <(&Equipped, &Armor)>::query()
                .iter(ecs)
                .filter(|(equipped, _)| equipped.owner == *victim)
                .map(|(_, armor)| armor.0)
                .sum();
            base_armor + equipped_armor
        } else { 0 };
        let final_damage = base_damage + weapon_damage - defense;

        if final_damage <= 0 {
            log.entries.push(format!("{} is unable to hurt {}", attacker_name, victim_name));
        }
        else if let Ok(mut health) = ecs.entry_mut(*victim)
            .unwrap().get_component_mut::<Health>()
        {
            log.entries.push(format!("{} hits {}, causing {} damage.", attacker_name, victim_name, final_damage));
            health.current -= final_damage;
            if health.current < 1 {
                if is_player {
                    *turn_state = TurnState::GameOver;
                }
                else {
                    log.entries.push(format!("{} is dead", victim_name));
                    commands.remove(*victim);
                }
            }
        }
        commands.remove(*message);
    });
}