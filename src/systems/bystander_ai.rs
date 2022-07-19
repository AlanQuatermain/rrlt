use crate::prelude::*;

#[system(for_each)]
#[read_component(Player)]
#[read_component(Point)]
#[filter(component::<Bystander>())]
pub fn bystander_ai(
    ecs: &SubWorld,
    entity: &Entity,
    pos: &Point,
    fov: &FieldOfView,
    name: &Name,
    quips: Option<&mut Quips>,
    #[resource] gamelog: &mut Gamelog,
    #[resource] rng: &mut RandomNumberGenerator,
    commands: &mut CommandBuffer,
) {
    let mut destination = *pos;
    match rng.roll_dice(1, 5) {
        1 => destination.x -= 1,
        2 => destination.x += 1,
        3 => destination.y -= 1,
        4 => destination.y += 1,
        _ => {}
    }

    if destination != *pos {
        commands.push((
            (),
            WantsToMove {
                entity: *entity,
                destination,
            },
        ));
    }

    if let Some(quips) = quips {
        let player_pos = <&Point>::query()
            .filter(component::<Player>())
            .iter(ecs)
            .nth(0)
            .unwrap();
        if !quips.0.is_empty()
            && fov.visible_tiles.contains(player_pos)
            && rng.roll_dice(1, 10) == 1
        {
            let quip_idx = rng.random_slice_index(quips.0.as_slice()).unwrap();
            gamelog
                .entries
                .push(format!("{} says \"{}\"", name.0, quips.0[quip_idx]));
            quips.0.remove(quip_idx);
        }
    }
}
