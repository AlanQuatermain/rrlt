use crate::prelude::*;

pub fn identify_entity(
    ecs: &SubWorld,
    effect: &EffectSpawner,
    target: Entity,
    dm: &mut MasterDungeonMap,
    commands: &mut CommandBuffer,
) {
    // Should always have a creator
    if effect.creator.is_none() {
        return;
    }

    let is_player = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .filter(|e| **e == effect.creator.unwrap())
        .nth(0)
        .is_some();

    if !is_player {
        // we don't care if someone else triggered identification
        return;
    }

    if let Ok(entry) = ecs.entry_ref(target) {
        if let Ok(name) = entry.get_component::<Name>() {
            if !dm.identified_items.contains(&name.0) && is_tag_magic(&name.0) {
                dm.identified_items.insert(name.0.clone());
            }
            commands.remove_component::<ObfuscatedName>(target);
        }
    }
}
