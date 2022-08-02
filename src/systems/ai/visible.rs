use crate::prelude::*;

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
    _pos: &Point,
    _name: &Name,
    fov: &FieldOfView,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
) {
    let mut reactions: Vec<(usize, Reaction, Entity)> = Vec::new();
    <(Entity, &Point, &Faction)>::query()
        .iter(ecs)
        .filter(|(e, p, _)| *e != entity && fov.visible_tiles.contains(p))
        .for_each(|(e, p, f)| {
            reactions.push((
                map.point2d_to_index(*p),
                faction_reaction(&faction.name, &f.name, &RAWS.lock().unwrap()),
                *e,
            ));
        });

    let mut flee: Vec<usize> = Vec::new();
    for reaction in reactions.iter() {
        match reaction.1 {
            Reaction::Attack => {
                commands.add_component(*entity, WantsToApproach { idx: reaction.0 });
                commands.add_component(*entity, Chasing { target: reaction.2 });
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
