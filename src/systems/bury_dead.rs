use crate::prelude::*;

#[system]
#[read_component(Pools)]
#[read_component(Player)]
#[read_component(Name)]
pub fn bury_dead(
    ecs: &mut SubWorld,
    #[resource] gamelog: &mut Gamelog,
    #[resource] turn_state: &mut TurnState,
    commands: &mut CommandBuffer,
) {
    let player_pools = <&Pools>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();
    if player_pools.hit_points.current <= 0 {
        *turn_state = TurnState::GameOver;
        return;
    }

    <(&Pools, &Name, Entity)>::query()
        .filter(!component::<Player>())
        .iter(ecs)
        .filter(|(pools, _, _)| pools.hit_points.current <= 0)
        .for_each(|(_, name, entity)| {
            gamelog.entries.push(format!("{} is dead!", name.0));
            commands.remove(*entity);
        });

    <Entity>::query()
        .filter(component::<Consumed>())
        .for_each(ecs, |entity| commands.remove(*entity));

    <Entity>::query()
        .filter(component::<EntityMoved>())
        .for_each(ecs, |entity| {
            commands.remove_component::<EntityMoved>(*entity)
        });
}
