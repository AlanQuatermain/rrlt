use crate::prelude::*;

#[system]
#[write_component(MyTurn)]
#[write_component(Initiative)]
#[read_component(Point)]
#[read_component(Attributes)]
#[read_component(Player)]
#[read_component(Pools)]
#[read_component(StatusEffect)]
#[write_component(Duration)]
#[read_component(Confusion)]
#[read_component(DamageOverTime)]
#[read_component(StatusEffect)]
pub fn initiative(
    ecs: &mut SubWorld,
    #[resource] rng: &mut RandomNumberGenerator,
    #[resource] turn_state: &mut TurnState,
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
        Option<&Pools>,
        Option<&Attributes>,
        Option<&Player>,
    )>::query()
    .for_each_mut(ecs, |(entity, initiative, pos, stats, attrs, player)| {
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

            // Apply encumbrance penalty
            if let Some(stats) = stats {
                initiative.current += f32::floor(stats.total_initiative_penalty) as i32;
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

    // Handle durations, if we've hit the player's turn
    if *turn_state == TurnState::AwaitingInput {
        <(
            Entity,
            &mut Duration,
            &StatusEffect,
            Option<&DamageOverTime>,
        )>::query()
        .for_each_mut(ecs, |(ent, duration, effect, maybe_dot)| {
            duration.0 -= 1;
            if let Some(dot) = maybe_dot {
                add_effect(
                    None,
                    EffectType::Damage { amount: dot.damage },
                    Targets::Single {
                        target: effect.target,
                    },
                );
            }
            if duration.0 < 1 {
                commands.add_component(effect.target, EquipmentChanged);
                commands.remove(*ent);
            }
        });
    }
}
