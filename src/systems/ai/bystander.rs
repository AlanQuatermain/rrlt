use crate::prelude::*;

#[system(for_each)]
#[read_component(Player)]
#[read_component(Point)]
#[filter(component::<Bystander>())]
pub fn bystander_ai(
    entity: &Entity,
    pos: &Point,
    _my_turn: &MyTurn,
    #[resource] rng: &mut RandomNumberGenerator,
    commands: &mut CommandBuffer,
) {
    let mut destination = *pos;
    match rng.roll_dice(1, 5) {
        1 => destination.x -= 1,
        2 => destination.x += 1,
        3 => destination.y -= 1,
        4 => destination.y += 1,
        _ => {}
    }

    if destination != *pos {
        commands.push((
            (),
            WantsToMove {
                entity: *entity,
                destination,
            },
        ));
    }
}
