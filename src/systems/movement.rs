use crate::prelude::*;

#[system(for_each)]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(Point)]
#[read_component(WantsToMove)]
#[write_component(FieldOfView)]
pub fn movement(
    entity: &Entity,
    want_move: &WantsToMove,
    fov: &mut FieldOfView,
    pos: &mut Point,
    player: Option<&Player>,
    #[resource] map: &mut Map,
    #[resource] camera: &mut Camera,
    commands: &mut CommandBuffer,
) {
    let to_idx = map.point2d_to_index(want_move.destination);
    if !crate::spatial::is_blocked(to_idx) {
        fov.is_dirty = true;
        commands.add_component(*entity, EntityMoved);

        let from_idx = map.point2d_to_index(*pos);
        crate::spatial::move_entity(*entity, from_idx, to_idx);
        *pos = want_move.destination;

        if player.is_some() {
            camera.on_player_move(want_move.destination);
        }
    }
    commands.remove_component::<WantsToMove>(*entity);
}

#[system(for_each)]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(Point)]
#[read_component(ApplyTeleport)]
#[write_component(FieldOfView)]
pub fn teleport(
    entity: &Entity,
    teleport: &ApplyTeleport,
    pos: &Point,
    player: Option<&Player>,
    #[resource] map: &Map,
    #[resource] turn_state: &mut TurnState,
    commands: &mut CommandBuffer,
) {
    if teleport.depth == map.depth {
        // Just move around the map
        commands.add_component(
            *entity,
            WantsToMove {
                destination: teleport.destination,
            },
        );
    }
    else if player.is_some() {
        *turn_state = TurnState::LevelTeleport {
            destination: teleport.destination,
            depth: teleport.depth,
        };
        return;
    }
    else {
        let from_idx = map.point2d_to_index(*pos);
        crate::spatial::remove_entity(*entity, from_idx);
        commands.add_component(
            *entity,
            OtherLevelPosition {
                position: teleport.destination,
                depth: teleport.depth,
            },
        );
    }
    commands.remove_component::<ApplyTeleport>(*entity);
}
