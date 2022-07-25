use crate::{prelude::*, systems::name_for};

#[system(for_each)]
#[read_component(MyTurn)]
#[read_component(Faction)]
#[read_component(Point)]
#[write_component(WantsToApproach)]
#[write_component(WantsToFlee)]
#[read_component(FieldOfView)]
#[read_component(Name)]
#[read_component(Player)]
#[filter(component::<MyTurn>()&!component::<Player>())]
pub fn visible(
    ecs: &mut SubWorld,
    entity: &Entity,
    faction: &Faction,
    pos: &Point,
    fov: &FieldOfView,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
) {
    let my_idx = map.point2d_to_index(*pos);
    let reactions: Vec<(usize, Reaction, Entity)> = <(&Point, &Faction, Entity)>::query()
        .iter(ecs)
        .filter(|(p, _, _)| fov.visible_tiles.contains(p))
        .map(|(pt, other_faction, other_entity)| {
            (
                map.point2d_to_index(*pt),
                faction_reaction(&faction.name, &other_faction.name, &RAWS.lock().unwrap()),
                *other_entity,
            )
        })
        .collect();

    let mut flee: Vec<usize> = Vec::new();
    for reaction in reactions.iter() {
        match reaction.1 {
            Reaction::Attack => {
                commands.add_component(*entity, WantsToApproach { idx: reaction.0 });
                commands.add_component(*entity, Chasing { target: reaction.2 });

                let me = name_for(entity, ecs).0;
                let them = name_for(&reaction.2, ecs).0;
                let target_pos = map.index_to_point2d(reaction.0);
                println!("{} is chasing {} at {:?}", me, them, target_pos);
                // This overrides any other concerns
                return;
            }
            Reaction::Flee => flee.push(reaction.0),
            _ => {}
        }
    }

    if !flee.is_empty() {
        commands.add_component(*entity, WantsToFlee { indices: flee });
    }
}
