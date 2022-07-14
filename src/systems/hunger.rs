use crate::prelude::*;

#[system]
#[write_component(HungerClock)]
#[write_component(Health)]
#[read_component(Player)]
pub fn hunger(
    ecs: &mut SubWorld,
    #[resource] turn_state: &TurnState,
    #[resource] gamelog: &mut Gamelog,
    commands: &mut CommandBuffer
) {
    let query = <(&mut HungerClock, Entity)>::query();
    if *turn_state == TurnState::PlayerTurn {
        query.filter(component::<Player>()).for_each_mut(ecs, |(clock, entity)| {
            update_hunger(true, clock, entity, gamelog, commands);
        });
    }
    else {
        query.filter(!component::<Player>()).for_each_mut(ecs, |(clock, entity)| {
            update_hunger(false, clock, entity, gamelog, commands);
        })
    }
}

fn update_hunger(
    is_player: bool,
    clock: &mut HungerClock,
    entity: &Entity,
    gamelog: &mut Gamelog,
    commands: &mut CommandBuffer
) {
    clock.duration -= 1;
    if clock.duration < 1 {
        match clock.state {
            HungerState::WellFed => {
                clock.state = HungerState::Normal;
                clock.duration = 200;
                if is_player {
                    gamelog.entries.push("You are no longer well-fed.".to_string());
                }
            },
            HungerState::Normal => {
                clock.state = HungerState::Hungry;
                clock.duration = 200;
                if is_player {
                    gamelog.entries.push("You are hungry.".to_string());
                }
            },
            HungerState::Hungry => {
                clock.state = HungerState::Starving;
                clock.duration = 200;
                if is_player {
                    gamelog.entries.push("You are starving!".to_string());
                }
            },
            HungerState::Starving => {
                // Inflict damage from hunger.
                if is_player {
                    gamelog.entries.push("Your hunger pangs are getting painful!".to_string());
                }
                commands.push(((), InflictDamage {
                    target: *entity,
                    user_entity: *entity,
                    damage: 1,
                    item_entity: None
                }));
            }
        }
    }
}