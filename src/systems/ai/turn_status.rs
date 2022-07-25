use crate::prelude::*;

#[system]
#[write_component(MyTurn)]
#[write_component(Confusion)]
pub fn turn_status(
    ecs: &mut SubWorld,
    #[resource] turn_state: &mut TurnState,
    commands: &mut CommandBuffer,
) {
    if *turn_state != TurnState::Ticking {
        return;
    }

    <(Entity, &mut Confusion)>::query()
        .filter(component::<MyTurn>())
        .for_each_mut(ecs, |(entity, confused)| {
            confused.0 -= 1;
            if confused.0 < 1 {
                commands.remove_component::<Confusion>(*entity);
            } else {
                commands.remove_component::<MyTurn>(*entity);
            }
        });
}
