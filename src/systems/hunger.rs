use crate::prelude::*;

#[system]
#[write_component(HungerClock)]
#[read_component(Player)]
pub fn hunger(ecs: &mut SubWorld, #[resource] gamelog: &mut Gamelog, commands: &mut CommandBuffer) {
    <(&mut HungerClock, Entity)>::query()
        .filter(component::<Player>() & component::<MyTurn>())
        .for_each_mut(ecs, |(clock, entity)| {
            update_hunger(clock, entity, gamelog, commands);
        });
}

fn update_hunger(
    clock: &mut HungerClock,
    entity: &Entity,
    gamelog: &mut Gamelog,
    commands: &mut CommandBuffer,
) {
    clock.duration -= 1;
    if clock.duration < 1 {
        match clock.state {
            HungerState::WellFed => {
                clock.state = HungerState::Normal;
                clock.duration = 200;
                gamelog
                    .entries
                    .push("You are no longer well-fed.".to_string());
            }
            HungerState::Normal => {
                clock.state = HungerState::Hungry;
                clock.duration = 200;
                gamelog.entries.push("You are hungry.".to_string());
            }
            HungerState::Hungry => {
                clock.state = HungerState::Starving;
                clock.duration = 200;
                gamelog.entries.push("You are starving!".to_string());
            }
            HungerState::Starving => {
                // Inflict damage from hunger.
                gamelog
                    .entries
                    .push("Your hunger pangs are getting painful!".to_string());
                add_effect(
                    None,
                    EffectType::Damage { amount: 1 },
                    Targets::Single { target: *entity },
                );
            }
        }
    }
}
