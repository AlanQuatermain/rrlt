use crate::prelude::*;

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
            if let Ok(fov) = entry.get_component::<FieldOfView>() {
                commands.add_component(want_move.entity, fov.clone_dirty());

                if entry.get_component::<Player>().is_ok() {}
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
