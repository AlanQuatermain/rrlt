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
pub fn trigger(ecs: &SubWorld, commands: &mut CommandBuffer, #[resource] gamelog: &mut Gamelog) {
    let moved_entities: Vec<(Entity, Point)> = <(Entity, &Point)>::query()
        .filter(component::<EntityMoved>())
        .iter(ecs)
        .map(|(e, p)| (*e, *p))
        .collect();

    let mut teleports: Vec<Entity> = Vec::new();
    for (entity, pos) in moved_entities {
        let is_player = ecs
            .entry_ref(entity)
            .unwrap()
            .get_component::<Player>()
            .is_ok();

        <(Entity, &Point, &Name, Option<&Damage>, Option<&TeleportTo>)>::query()
            .filter(component::<EntryTrigger>())
            .iter(ecs)
            .filter(|(_, p, _, _, _)| pos == **p)
            .for_each(|(trigger_entity, _, trigger_name, damage, teleport)| {
                commands.remove_component::<Hidden>(*trigger_entity);
                if let Some(damage) = damage {
                    gamelog
                        .entries
                        .push(format!("{} triggers!", trigger_name.0));
                    commands.push((
                        (),
                        InflictDamage {
                            target: entity,
                            user_entity: *trigger_entity,
                            damage: damage.0,
                            item_entity: None,
                        },
                    ));
                } else if let Some(teleport) = teleport {
                    if !teleport.player_only || is_player {
                        commands.add_component(
                            entity,
                            ApplyTeleport {
                                destination: teleport.position,
                                depth: teleport.depth,
                            },
                        );
                        teleports.push(entity);
                    }
                }
            });
    }

    for entity in teleports {
        if ecs
            .entry_ref(entity)
            .unwrap()
            .get_component::<SingleActivation>()
            .is_ok()
        {
            commands.remove(entity);
        }
    }
}
