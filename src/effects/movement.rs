use crate::prelude::*;

pub fn apply_teleport(
    ecs: &mut SubWorld,
    destination: &EffectSpawner,
    target: Entity,
    commands: &mut CommandBuffer,
) {
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();
    if let EffectType::TeleportTo {
        pos,
        depth,
        player_only,
    } = &destination.effect_type
    {
        if !player_only || target == *player_entity {
            commands.add_component(
                target,
                ApplyTeleport {
                    destination: *pos,
                    depth: *depth,
                },
            )
        }
    }
}
