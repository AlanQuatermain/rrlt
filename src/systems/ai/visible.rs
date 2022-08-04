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
#[filter(component::<MyTurn>()&!component::<Player>())]
pub fn visible(
    ecs: &mut SubWorld,
    entity: &Entity,
    faction: &Faction,
    pos: &Point,
    _name: &Name,
    fov: &FieldOfView,
    stats: &Pools,
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

    let mut flee: Vec<usize> = Vec::new();
    for reaction in reactions.iter() {
        match reaction.1 {
            Reaction::Attack => {
                if let Some(abilities) = abilities {
                    let end = map.index_to_point2d(reaction.0);
                    let range = DistanceAlg::Pythagoras.distance2d(*pos, end);
                    for ability in abilities.abilities.iter() {
                        if range >= ability.min_range
                            && range <= ability.range
                            && rng.roll_dice(1, 100) >= (ability.chance * 100.0) as i32
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
