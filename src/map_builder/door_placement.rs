use super::MetaMapBuilder;
use crate::prelude::*;

#[derive(Default)]
pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.doors(rng, build_data);
    }
}

impl DoorPlacement {
    #[allow(dead_code)]
    pub fn new() -> Box<DoorPlacement> {
        Box::new(DoorPlacement::default())
    }

    fn doors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(halls_original) = &build_data.corridors {
            let halls = halls_original.clone();
            for hall in halls.iter() {
                if hall.len() > 2 {
                    if self.door_possible(build_data, hall[0]) {
                        build_data
                            .spawn_list
                            .push((build_data.map.index_to_point2d(hall[0]), "Door".to_string()));
                    }
                }
            }
        } else {
            let tiles = build_data.map.tiles.clone();
            for (i, tile) in tiles.iter().enumerate() {
                if *tile == TileType::Floor
                    && self.door_possible(build_data, i)
                    && rng.roll_dice(1, 3) == 1
                {
                    build_data
                        .spawn_list
                        .push((build_data.map.index_to_point2d(i), "Door".to_string()));
                }
            }
        }
    }

    fn door_possible(&self, build_data: &mut BuilderMap, idx: usize) -> bool {
        let pos = build_data.map.index_to_point2d(idx);
        for spawn in build_data.spawn_list.iter() {
            if spawn.0 == pos {
                return false;
            }
        }

        // Check for east-west door possibility
        if (build_data.map.tiles[idx] == TileType::Floor)
            && (pos.x > 1 && build_data.map.tiles[idx - 1] == TileType::Floor)
            && (pos.x < build_data.map.width as i32 - 2
                && build_data.map.tiles[idx + 1] == TileType::Floor)
            && (pos.y > 1 && build_data.map.tiles[idx - build_data.map.width] == TileType::Wall)
            && (pos.y < build_data.map.height as i32 - 2
                && build_data.map.tiles[idx + build_data.map.width] == TileType::Wall)
        {
            return true;
        }

        // Check for north-south door possibility
        if (build_data.map.tiles[idx] == TileType::Floor)
            && (pos.x > 1 && build_data.map.tiles[idx - 1] == TileType::Wall)
            && (pos.x < build_data.map.width as i32 - 2
                && build_data.map.tiles[idx + 1] == TileType::Wall)
            && (pos.y > 1 && build_data.map.tiles[idx - build_data.map.width] == TileType::Floor)
            && (pos.y < build_data.map.height as i32 - 2
                && build_data.map.tiles[idx + build_data.map.width] == TileType::Floor)
        {
            return true;
        }

        false
    }
}
