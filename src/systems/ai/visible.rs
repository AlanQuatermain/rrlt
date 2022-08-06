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
#[read_component(SpecialAbilities)]
#[read_component(SpellTemplate)]
#[read_component(Weapon)]
#[read_component(Equipped)]
#[filter(component::<MyTurn>()&!component::<Player>())]
pub fn visible(
    ecs: &mut SubWorld,
    entity: &Entity,
    faction: &Faction,
    pos: &Point,
    _name: &Name,
    fov: &FieldOfView,
    stats: &Pools,
    movement: &MoveMode,
    abilities: Option<&SpecialAbilities>,
    #[resource] map: &Map,
    #[resource] rng: &mut RandomNumberGenerator,
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

    let mut flee: Vec<usize> = Vec::new();
    for reaction in reactions.iter() {
        match reaction.1 {
            Reaction::Attack => {
                let end = map.index_to_point2d(reaction.0);
                let range = DistanceAlg::Pythagoras.distance2d(*pos, end);

                if let Some(abilities) = abilities {
                    for ability in abilities.abilities.iter() {
                        if range >= ability.min_range
                            && range <= ability.range
                            && rng.roll_dice(1, 100) <= (ability.chance * 100.0) as i32
                        {
                            let spell = find_spell_entity(ecs, &ability.spell).unwrap();
                            let spell_entry = ecs.entry_ref(spell).unwrap();
                            let template = spell_entry.get_component::<SpellTemplate>().unwrap();
                            if stats.mana.current >= template.mana_cost {
                                commands.add_component(
                                    *entity,
                                    WantsToCastSpell {
                                        spell: find_spell_entity(ecs, &ability.spell).unwrap(),
                                        target: Some(end),
                                    },
                                );
                                return;
                            }
                        }
                    }
                }

                // Look at equipped weapons--any ranged in there?
                for (_, equipped, wpn_range) in weaponry.iter() {
                    if *equipped && wpn_range.is_some() {
                        if wpn_range.unwrap() >= range as i32 {
                            commands.add_component(*entity, WantsToShoot { target: reaction.2 });
                            return;
                        }
                    }
                }

                // Any available ranged weapons we might switch to?
                for (wpn, equipped, wpn_range) in weaponry.iter() {
                    if !*equipped && wpn_range.is_some() {
                        if wpn_range.unwrap() >= range as i32 {
                            // Can use this, let's equip it
                            commands.add_component(
                                *wpn,
                                UseItem {
                                    user: *entity,
                                    target: None,
                                },
                            );
                            return;
                        }
                    }
                }

                if movement.0 != Movement::Immobile {
                    commands.add_component(*entity, WantsToApproach { idx: reaction.0 });
                    commands.add_component(*entity, Chasing { target: reaction.2 });
                    // This overrides any other concerns
                }
                return;
            }
            Reaction::Flee => flee.push(reaction.0),
            _ => {}
        }
    }

    if !flee.is_empty() && movement.0 != Movement::Immobile {
        commands.add_component(*entity, WantsToFlee { indices: flee });
    }
}
