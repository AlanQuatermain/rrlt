use crate::prelude::*;

#[system]
#[read_component(EntityMoved)]
#[read_component(Point)]
#[read_component(EntryTrigger)]
#[read_component(Name)]
#[write_component(Hidden)]
#[read_component(Damage)]
pub fn trigger(ecs: &SubWorld, commands: &mut CommandBuffer, #[resource] gamelog: &mut Gamelog) {
    let moved_entities: Vec<(Entity, Point)> = <(Entity, &Point)>::query()
        .filter(component::<EntityMoved>())
        .iter(ecs)
        .map(|(e, p)| (*e, *p))
        .collect();

    for (entity, pos) in moved_entities {
        <(Entity, &Point, &Name, &Damage)>::query()
            .filter(component::<EntryTrigger>())
            .iter(ecs)
            .filter(|(_, p, _, _)| pos == **p)
            .for_each(|(trigger_entity, _, trigger_name, damage)| {
                gamelog
                    .entries
                    .push(format!("{} triggers!", trigger_name.0));
                commands.remove_component::<Hidden>(*trigger_entity);
                commands.push((
                    (),
                    InflictDamage {
                        target: entity,
                        user_entity: *trigger_entity,
                        damage: damage.0,
                        item_entity: None,
                    },
                ));
            });
    }
}
