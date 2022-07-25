use lazy_static::__Deref;

use crate::prelude::*;

#[system]
#[write_component(MyTurn)]
#[write_component(Initiative)]
#[read_component(Point)]
#[read_component(Attributes)]
#[read_component(Player)]
pub fn initiative(
    ecs: &mut SubWorld,
    #[resource] rng: &mut RandomNumberGenerator,
    #[resource] turn_state: &mut TurnState,
    // #[state] time_ms: &mut std::time::Instant,
    commands: &mut CommandBuffer,
) {
    if *turn_state != TurnState::Ticking {
        return;
    }

    let player_pos = <&Point>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap()
        .clone();

    // Clear any MyTurn left attached by mistake
    <Entity>::query()
        .filter(component::<MyTurn>())
        .for_each(ecs, |entity| {
            commands.remove_component::<MyTurn>(*entity);
        });

    // Roll for initiative!
    <(
        Entity,
        &mut Initiative,
        &Point,
        Option<&Attributes>,
        Option<&Player>,
    )>::query()
    .for_each_mut(ecs, |(entity, initiative, pos, attrs, player)| {
        initiative.current -= 1;
        if initiative.current < 1 {
            // It's my turn!
            let mut my_turn = true;

            // Re-roll
            initiative.current = 6 + rng.roll_dice(1, 6);

            // Give a bonus for quickness.
            if let Some(attrs) = attrs {
                initiative.current -= attrs.quickness.bonus;
            }

            // TODO: More initiative-granting boosts/penalties will go here later.

            // If it's the player, we want to go to an AwaitingInput state.
            if player.is_some() {
                *turn_state = TurnState::AwaitingInput;
            } else {
                // Ignore entities too far away from the player
                let distance = DistanceAlg::Pythagoras.distance2d(player_pos, *pos);
                if distance > 10.0 {
                    // println!("Doing nothing: {:?}", *entity);
                    my_turn = false;
                }
            }

            if my_turn {
                commands.add_component(*entity, MyTurn);
            }
        }
    });
}
