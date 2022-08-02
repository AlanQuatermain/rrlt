use crate::prelude::*;
use crate::raws::Reaction;

#[system(for_each)]
#[write_component(MyTurn)]
#[read_component(Faction)]
#[read_component(Point)]
#[write_component(WantsToAttack)]
#[read_component(Name)]
#[filter(component::<MyTurn>()&!component::<Player>())]
pub fn adjacent(
    ecs: &SubWorld,
    entity: &Entity,
    faction: &Faction,
    pos: &Point,
    _name: &Name,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
) {
    // Add possible reactions to adjacent entities for each direction.
    let adjacent_pts = adjacent_points(pos, map);
    let mut reactions: Vec<(Entity, Reaction)> = Vec::new();

    <(Entity, &Point, &Faction)>::query()
        .iter(ecs)
        .filter(|(_, p, _)| adjacent_pts.contains(p))
        .for_each(|(e, _, f)| {
            reactions.push((
                *e,
                faction_reaction(&faction.name, &f.name, &RAWS.lock().unwrap()),
            ));
        });

    let mut acted = false;
    for reaction in reactions.iter() {
        if let Reaction::Attack = reaction.1 {
            commands.push((
                (),
                WantsToAttack {
                    attacker: *entity,
                    victim: reaction.0,
                },
            ));
            acted = true;
            break;
        }
    }

    if acted {
        commands.remove_component::<MyTurn>(*entity);
    }
}

fn adjacent_points(pos: &Point, map: &Map) -> Vec<Point> {
    let mut points = Vec::new();

    for x in -1..=1 {
        for y in -1..=1 {
            if x == 0 && y == 0 {
                continue;
            }
            let pt = *pos + Point::new(x, y);
            if map.in_bounds(pt) {
                points.push(pt);
            }
        }
    }

    points
}
