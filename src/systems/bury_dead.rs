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
#[read_component(StatusEffect)]
#[read_component(OnDeath)]
#[read_component(SpellTemplate)]
pub fn bury_dead(
    ecs: &mut SubWorld,
    #[resource] turn_state: &mut TurnState,
    #[resource] rng: &mut RandomNumberGenerator,
    #[resource] dm: &MasterDungeonMap,
    #[resource] map: &Map,
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
            crate::gamelog::Logger::new()
                .npc_name(&name.0)
                .color(RED)
                .append("is dead!")
                .log();
            dead_list.insert(*entity, *pos);
        });

    // Find any effects applying to dead entities and remove them.
    <(&StatusEffect, Entity)>::query()
        .iter(ecs)
        .filter(|(st, _)| dead_list.contains_key(&st.target))
        .for_each(|(_, e)| commands.remove(*e));

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
                spawn_named_item(
                    raws,
                    &drop,
                    SpawnType::AtPosition { point: *pos },
                    dm,
                    commands,
                );
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

    <(Entity, &OnDeath)>::query()
        .iter(ecs)
        .filter_map(|(e, od)| dead_list.get(e).map(|p| (*p, od.clone())))
        .for_each(|(pos, od)| {
            for effect in od.abilities.iter() {
                if rng.roll_dice(1, 100) <= (effect.chance * 100.0) as i32 {
                    let spell_entity = find_spell_entity(ecs, &effect.spell).unwrap();
                    let tile_idx = map.point2d_to_index(pos);
                    let targets = if let Ok(aoe) = ecs
                        .entry_ref(spell_entity)
                        .unwrap()
                        .get_component::<AreaOfEffect>()
                    {
                        Targets::Tiles {
                            tiles: aoe_tiles(map, pos, aoe.0),
                        }
                    } else {
                        Targets::Tile { tile_idx }
                    };

                    add_effect(
                        None,
                        EffectType::CastSpell {
                            spell: spell_entity,
                        },
                        targets,
                    );
                }
            }
        });

    // Do deletions last
    for (entity, _) in dead_list {
        commands.remove(entity);
    }
}
