use std::collections::HashSet;

use crate::prelude::*;
use crate::raws::Reaction;

#[system(for_each)]
#[write_component(MyTurn)]
#[read_component(Faction)]
#[read_component(Point)]
#[write_component(WantsToAttack)]
#[read_component(Name)]
#[read_component(TileSize)]
#[read_component(Weapon)]
#[read_component(Carried)]
#[read_component(Equipped)]
#[filter(component::<MyTurn>()&!component::<Player>())]
pub fn adjacent(
    ecs: &SubWorld,
    entity: &Entity,
    faction: &Faction,
    pos: &Point,
    size: Option<&TileSize>,
    _name: &Name,
    #[resource] map: &Map,
    commands: &mut CommandBuffer,
) {
    // Add possible reactions to adjacent entities for each direction.
    let adjacent_pts = adjacent_points(pos, size, map);
    let mut reactions: Vec<(Entity, Reaction)> = Vec::new();

    <(Entity, &Point, &Faction, Option<&TileSize>)>::query()
        .iter(ecs)
        .filter(|(_, p, _, size)| {
            let mut points = HashSet::new();
            if let Some(size) = size {
                points.extend(Rect::with_size(p.x, p.y, size.x, size.y).point_set());
            } else {
                points.insert(**p);
            }
            adjacent_pts.intersection(&points).count() != 0
        })
        .for_each(|(e, _, f, _)| {
            reactions.push((
                *e,
                faction_reaction(&faction.name, &f.name, &RAWS.lock().unwrap()),
            ));
        });

    // Cache available weaponry
    let weaponry: Vec<_> = <(Entity, &Weapon, &Carried, Option<&Equipped>)>::query()
        .iter(ecs)
        .filter_map(|(e, w, c, eq)| {
            if c.0 == *entity {
                Some((*e, eq.is_some(), w.range))
            } else {
                None
            }
        })
        .collect();

    let mut acted = false;
    for reaction in reactions.iter() {
        if let Reaction::Attack = reaction.1 {
            for (wpn_entity, equipped, range) in weaponry.iter() {
                if *equipped && range.is_some() {
                    // ranged weapon equipped, need to switch to melee
                    commands.add_component(
                        *wpn_entity,
                        UseItem {
                            user: *entity,
                            target: None,
                        },
                    );
                    acted = true;
                    break;
                }
            }

            // attack either with equipped melee weapon or natural weapon
            if !acted {
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
    }

    if acted {
        commands.remove_component::<MyTurn>(*entity);
    }
}

fn adjacent_points(pos: &Point, maybe_size: Option<&TileSize>, map: &Map) -> HashSet<Point> {
    let mut points = HashSet::new();

    if let Some(size) = maybe_size {
        let base_rect = Rect::with_size(pos.x, pos.y, size.x, size.y);
        // expand by 1 in every direction
        let expanded_rect = Rect::with_exact(
            base_rect.x1 - 1,
            base_rect.y1 - 1,
            base_rect.x2 + 1,
            base_rect.y2 + 1,
        );
        return &expanded_rect.point_set() - &base_rect.point_set();
    } else {
        for x in -1..=1 {
            for y in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }
                let pt = *pos + Point::new(x, y);
                if map.in_bounds(pt) {
                    points.insert(pt);
                }
            }
        }
    }
    points
}
