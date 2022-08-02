use std::collections::HashSet;

use crate::prelude::*;

#[system]
#[write_component(MyTurn)]
#[write_component(Confusion)]
#[read_component(StatusEffect)]
pub fn turn_status(
    ecs: &mut SubWorld,
    #[resource] turn_state: &mut TurnState,
    commands: &mut CommandBuffer,
) {
    if *turn_state != TurnState::Ticking {
        return;
    }

    let statuses: Vec<_> = <(Entity, &StatusEffect)>::query()
        .iter(ecs)
        .map_while(|(ent, eff)| ecs.entry_ref(*ent).ok().map(|v| (v, eff.clone())))
        .collect();
    let active_entities: HashSet<_> = <Entity>::query()
        .filter(component::<MyTurn>())
        .iter(ecs)
        .map(|e| *e)
        .collect();

    for (entry, effect) in statuses {
        if !active_entities.contains(&effect.target) {
            continue;
        }
        if entry.get_component::<Confusion>().is_ok() {
            commands.remove_component::<MyTurn>(effect.target);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: to_cp437('?'),
                    color: ColorPair::new(CYAN, BLACK),
                    lifespan: 200.0,
                },
                Targets::Single {
                    target: effect.target,
                },
            );
        }
    }
}
