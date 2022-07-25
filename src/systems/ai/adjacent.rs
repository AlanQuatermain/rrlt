use crate::prelude::*;
use crate::raws::Reaction;

#[system(for_each)]
#[write_component(MyTurn)]
#[read_component(Faction)]
#[read_component(Point)]
#[write_component(WantsToAttack)]
#[filter(component::<MyTurn>())]
pub fn adjacent(
    ecs: &SubWorld,
    entity: &Entity,
    faction: &Faction,
    pos: &Point,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
) {
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();

    if entity == player_entity {
        return;
    }

    // Add possible reactions to adjacents for each direction.
    let adjacent_pts = adjacent_points(pos, map);
    let reactions: Vec<(Entity, Reaction)> = <(&Point, &Faction, Entity)>::query()
        .iter(ecs)
        .filter(|(p, _, _)| adjacent_pts.contains(p))
        .map(|(_, other_faction, other_entity)| {
            (
                *other_entity,
                faction_reaction(&faction.name, &other_faction.name, &RAWS.lock().unwrap()),
            )
        })
        .collect();

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

    for x in -1..1 {
        for y in -1..1 {
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
