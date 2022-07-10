use crate::prelude::*;

#[system]
#[read_component(Player)]
#[read_component(Point)]
pub fn end_turn(
    ecs: &SubWorld,
    #[resource] turn_state: &mut TurnState
) {
    let current_state = turn_state.clone();
    let new_state = match turn_state {
        TurnState::AwaitingInput => return,
        TurnState::PlayerTurn => TurnState::MonsterTurn,
        TurnState::MonsterTurn => TurnState::AwaitingInput,
        _ => current_state
    };
    *turn_state = new_state;
}