use crate::prelude::*;
use crate::systems::name_for;

#[system(for_each)]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(Confusion)]
#[read_component(Name)]
#[read_component(Point)]
#[read_component(Hidden)]
pub fn movement(
    entity: &Entity,
    want_move: &WantsToMove,
    #[resource] map: &mut Map,
    #[resource] camera: &mut Camera,
    #[resource] gamelog: &mut Gamelog,
    #[resource] particle_builder: &mut ParticleBuilder,
    #[resource] rng: &mut RandomNumberGenerator,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    if map.can_enter_tile(want_move.destination) {
        let mut will_move = true;
        if let Ok(entry) = ecs.entry_ref(want_move.entity) {
            if let Ok(confusion) = entry.get_component::<Confusion>() {
                // entity is confused, and will not move
                will_move = false;
                if let Ok(current_pos) = entry.get_component::<Point>() {
                    particle_builder.request(
                        *current_pos,
                        ColorPair::new(MAGENTA, BLACK),
                        to_cp437('?'),
                        200.0,
                    );
                }

                // decrement the confusion duration
                let new_duration = confusion.0 - 1;
                if new_duration > 0 {
                    commands.add_component(want_move.entity, Confusion(new_duration));
                } else {
                    let name = name_for(&want_move.entity, ecs).0;
                    gamelog
                        .entries
                        .push(format!("{} shakes off their confusion.", name));
                    commands.remove_component::<Confusion>(want_move.entity);
                }
            } else if let Ok(fov) = entry.get_component::<FieldOfView>() {
                commands.add_component(want_move.entity, fov.clone_dirty());

                if entry.get_component::<Player>().is_ok() {
                    camera.on_player_move(want_move.destination);
                    fov.visible_tiles.iter().for_each(|pos| {
                        let idx = map.point2d_to_index(*pos);
                        map.revealed_tiles[idx] = true;

                        // Chance to find hidden things.
                        <(Entity, &Point, &Name)>::query()
                            .filter(component::<Hidden>())
                            .iter(ecs)
                            .filter(|(_, p, _)| *p == pos)
                            .for_each(|(entity, _, name)| {
                                if rng.roll_dice(1, 24) == 1 {
                                    gamelog.entries.push(format!("You spotted a {}.", name.0));
                                    commands.remove_component::<Hidden>(*entity);
                                }
                            });
                    });
                }
            }
        }
        if will_move {
            commands.add_component(want_move.entity, want_move.destination);
            commands.add_component(want_move.entity, EntityMoved);
            let idx = map.point2d_to_index(want_move.destination);
            map.blocked[idx] = true;
        }
    }
    commands.remove(*entity);
}
