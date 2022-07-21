use crate::prelude::*;
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps: HashMap<i32, Map>,
}

impl MasterDungeonMap {
    pub fn new() -> MasterDungeonMap {
        MasterDungeonMap::default()
    }

    pub fn store_map(&mut self, map: &Map) {
        self.maps.insert(map.depth, map.clone());
    }

    pub fn get_map(&self, depth: i32) -> Option<Map> {
        self.maps.get(&depth).map(|m| m.clone())
    }
}

pub fn freeze_level_entities(ecs: &World, depth: i32, commands: &mut CommandBuffer) {
    <(Entity, &Point)>::query()
        .filter(!component::<Player>() & !component::<OtherLevelPosition>())
        .for_each(ecs, |(entity, pos)| {
            commands.add_component(
                *entity,
                OtherLevelPosition {
                    position: pos.clone(),
                    depth,
                },
            );
            commands.remove_component::<Point>(*entity);
        });
}

pub fn thaw_level_entities(ecs: &World, depth: i32, commands: &mut CommandBuffer) {
    <(Entity, &OtherLevelPosition)>::query()
        .iter(ecs)
        .filter(|(_, opos)| opos.depth == depth)
        .for_each(|(entity, opos)| {
            commands.add_component(*entity, opos.position.clone());
            commands.remove_component::<OtherLevelPosition>(*entity);
        });
}
