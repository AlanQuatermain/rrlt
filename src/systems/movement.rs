use crate::prelude::*;
use crate::systems::name_for;

#[system(for_each)]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(Confusion)]
#[read_component(Name)]
#[read_component(Point)]
pub fn movement(
    entity: &Entity,
    want_move: &WantsToMove,
    #[resource] map: &mut Map,
    #[resource] camera: &mut Camera,
    #[resource] gamelog: &mut Gamelog,
    #[resource] particle_builder: &mut ParticleBuilder,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer
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
                        200.0
                    );
                }

                // decrement the confusion duration
                let new_duration = confusion.0 - 1;
                if new_duration > 0 {
                    commands.add_component(want_move.entity, Confusion(new_duration));
                }
                else {
                    let name = name_for(&want_move.entity, ecs).0;
                    gamelog.entries.push(format!("{} shakes off their confusion.", name));
                    commands.remove_component::<Confusion>(want_move.entity);
                }
            }
            else if let Ok(fov) = entry.get_component::<FieldOfView>() {
                commands.add_component(want_move.entity, fov.clone_dirty());

                if entry.get_component::<Player>().is_ok() {
                    camera.on_player_move(want_move.destination);
                    fov.visible_tiles.iter().for_each(|pos| {
                        map.revealed_tiles[map_idx(pos.x, pos.y)] = true;
                    });
                }
            }
        }
        if will_move {
            commands.add_component(want_move.entity, want_move.destination);
        }
    }
    commands.remove(*entity);
}