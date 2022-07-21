use std::collections::HashMap;

use crate::prelude::*;

#[system]
#[read_component(Pools)]
#[read_component(Player)]
#[read_component(Name)]
#[read_component(Carried)]
#[read_component(Equipped)]
#[read_component(LootTable)]
#[read_component(Point)]
pub fn bury_dead(
    ecs: &mut SubWorld,
    #[resource] gamelog: &mut Gamelog,
    #[resource] turn_state: &mut TurnState,
    #[resource] rng: &mut RandomNumberGenerator,
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

    let mut dead_list = HashMap::new();
    <(&Pools, &Name, &Point, Entity)>::query()
        .filter(!component::<Player>())
        .iter(ecs)
        .filter(|(pools, _, _, _)| pools.hit_points.current <= 0)
        .for_each(|(_, name, pos, entity)| {
            gamelog.entries.push(format!("{} is dead!", name.0));
            dead_list.insert(*entity, *pos);
        });

    // Have everything carried by dead entities drop to the ground,
    // and potentially spawn items from loot tables
    <(Entity, &Carried)>::query()
        .iter(ecs)
        .filter_map(|(e, c)| dead_list.get(&c.0).map(|p| (e, p)))
        .for_each(|(entity, pos)| {
            // place on the floor
            commands.add_component(*entity, pos.clone());
            // remove carry/equip status
            commands.remove_component::<Carried>(*entity);
            commands.remove_component::<Equipped>(*entity);
        });

    // Spawn any loot drops
    <(Entity, &LootTable)>::query()
        .iter(ecs)
        .filter_map(|(e, t)| dead_list.get(e).map(|p| (t, p)))
        .for_each(|(table, pos)| {
            let raws = &RAWS.lock().unwrap();
            if let Some(drop) = get_drop_item(raws, rng, &table.0) {
                spawn_named_item(raws, &drop, SpawnType::AtPosition { point: *pos }, commands);
            }
        });

    <Entity>::query()
        .filter(component::<Consumed>())
        .for_each(ecs, |entity| commands.remove(*entity));

    <Entity>::query()
        .filter(component::<EntityMoved>())
        .for_each(ecs, |entity| {
            commands.remove_component::<EntityMoved>(*entity)
        });

    // Do deletions last
    for (entity, _) in dead_list {
        commands.remove(entity);
    }
}
