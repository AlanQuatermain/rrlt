use crate::prelude::*;

#[system]
#[read_component(EntityMoved)]
#[read_component(Point)]
#[read_component(EntryTrigger)]
#[read_component(Name)]
#[write_component(Hidden)]
#[read_component(Damage)]
#[read_component(TeleportTo)]
#[read_component(SingleActivation)]
#[read_component(Player)]
pub fn trigger(
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
    #[resource] gamelog: &mut Gamelog,
    #[resource] map: &Map,
) {
    let moved_entities: Vec<(Entity, Point)> = <(Entity, &Point)>::query()
        .filter(component::<EntityMoved>())
        .iter(ecs)
        .map(|(e, p)| (*e, *p))
        .collect();

    let mut teleports: Vec<Entity> = Vec::new();
    for (entity, pos) in moved_entities {
        // Remove the movement marker
        commands.remove_component::<EntityMoved>(entity);

        <(Entity, &Point, &Name)>::query()
            .filter(component::<EntryTrigger>())
            .iter(ecs)
            .filter(|(_, p, _)| pos == **p)
            .for_each(|(trigger_entity, _, trigger_name)| {
                gamelog
                    .entries
                    .push(format!("{} triggers!", trigger_name.0));

                // add into the effects system
                let tile_idx = map.point2d_to_index(pos);
                add_effect(
                    Some(entity),
                    EffectType::TriggerFire {
                        trigger: *trigger_entity,
                    },
                    Targets::Tile { tile_idx },
                )
            });
    }
}
