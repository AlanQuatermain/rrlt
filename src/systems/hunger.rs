use crate::prelude::*;

#[system]
#[write_component(HungerClock)]
#[read_component(Player)]
pub fn hunger(ecs: &mut SubWorld, commands: &mut CommandBuffer) {
    <(&mut HungerClock, Entity)>::query()
        .filter(component::<Player>() & component::<MyTurn>())
        .for_each_mut(ecs, |(clock, entity)| {
            update_hunger(clock, entity, commands);
        });
}

fn update_hunger(clock: &mut HungerClock, entity: &Entity, _commands: &mut CommandBuffer) {
    clock.duration -= 1;
    if clock.duration < 1 {
        match clock.state {
            HungerState::WellFed => {
                clock.state = HungerState::Normal;
                clock.duration = 200;
                crate::gamelog::Logger::new()
                    .color(ORANGE)
                    .append("You are no longer well-fed.")
                    .log();
            }
            HungerState::Normal => {
                clock.state = HungerState::Hungry;
                clock.duration = 200;
                crate::gamelog::Logger::new()
                    .color(ORANGE)
                    .append("You are hungry.")
                    .log();
            }
            HungerState::Hungry => {
                clock.state = HungerState::Starving;
                clock.duration = 200;
                crate::gamelog::Logger::new()
                    .color(RED)
                    .append("You are starving!")
                    .log();
            }
            HungerState::Starving => {
                // Inflict damage from hunger.
                crate::gamelog::Logger::new()
                    .color(ORANGE)
                    .append("Your hunger pangs are getting painful! You suffer 1 hp damage.")
                    .log();
                add_effect(
                    None,
                    EffectType::Damage { amount: 1 },
                    Targets::Single { target: *entity },
                );
            }
        }
    }
}
