use crate::map_builder::common::{paint, Symmetry};
use crate::prelude::*;

const NUM_TILES: usize = MAP_WIDTH * MAP_HEIGHT;

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

pub struct DrunkardsWalkBuilder {
    pub spawn_mode: DrunkSpawnMode,
    pub lifetime: usize,
    pub floor_percent: f32,
    pub brush_size: i32,
    pub symmetry: Symmetry,
}

impl InitialMapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Wall);
        self.build(rng, build_data);
    }
}

impl DrunkardsWalkBuilder {
    #[allow(dead_code)]
    pub fn open_area() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            spawn_mode: DrunkSpawnMode::StartingPoint,
            lifetime: 400,
            floor_percent: 0.5,
            brush_size: 1,
            symmetry: Symmetry::None,
        })
    }

    #[allow(dead_code)]
    pub fn open_halls() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 400,
            floor_percent: 0.5,
            brush_size: 1,
            symmetry: Symmetry::None,
        })
    }

    #[allow(dead_code)]
    pub fn winding_passages() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 100,
            floor_percent: 0.4,
            brush_size: 1,
            symmetry: Symmetry::None,
        })
    }

    #[allow(dead_code)]
    pub fn fat_passages() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 100,
            floor_percent: 0.4,
            brush_size: 2,
            symmetry: Symmetry::None,
        })
    }

    #[allow(dead_code)]
    pub fn fearful_symmetry() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 100,
            floor_percent: 0.4,
            brush_size: 1,
            symmetry: Symmetry::Both,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Set a central starting point.
        let starting_position = Point::new(build_data.map.width / 2, build_data.map.height / 2);
        let start_idx = build_data.map.point2d_to_index(starting_position);
        build_data.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut digger_count = 0;
        while floor_tile_count < desired_tiles {
            let mut did_something = false;
            let mut drunk_pos = match self.spawn_mode {
                DrunkSpawnMode::StartingPoint => starting_position.clone(),
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        starting_position.clone()
                    } else {
                        Point::new(
                            rng.roll_dice(1, build_data.map.width as i32 - 3) + 1,
                            rng.roll_dice(1, build_data.map.height as i32 - 3) + 1,
                        )
                    }
                }
            };
            let mut drunk_life = self.lifetime;

            while drunk_life > 0 {
                let drunk_idx = build_data.map.point2d_to_index(drunk_pos);
                if build_data.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                paint(
                    &mut build_data.map,
                    self.symmetry,
                    self.brush_size,
                    drunk_pos,
                );
                build_data.map.tiles[drunk_idx] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => {
                        if drunk_pos.x > 2 {
                            drunk_pos.x -= 1
                        }
                    }
                    2 => {
                        if drunk_pos.x < build_data.map.width as i32 - 2 {
                            drunk_pos.x += 1
                        }
                    }
                    3 => {
                        if drunk_pos.y > 2 {
                            drunk_pos.y -= 1
                        }
                    }
                    _ => {
                        if drunk_pos.y < build_data.map.height as i32 - 2 {
                            drunk_pos.y += 1
                        }
                    }
                }

                drunk_life -= 1;
            }
            if did_something {
                build_data.take_snapshot();
            }

            digger_count += 1;
            for t in build_data.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = build_data
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }
    }
}
