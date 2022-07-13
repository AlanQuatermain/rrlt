use std::collections::HashMap;
use crate::prelude::*;
use super::MapArchitect;
use super::MAX_SPAWNS_PER_ROOM;

const NUM_TILES: usize = MAP_WIDTH * MAP_HEIGHT;

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode { Center, Random }

pub struct DrunkardsWalkArchitect {
    pub spawn_mode: DrunkSpawnMode,
    pub lifetime: usize,
    pub floor_percent: f32
}

impl Default for DrunkardsWalkArchitect {
    fn default() -> Self {
        Self {
            spawn_mode: DrunkSpawnMode::Center,
            lifetime: 400,
            floor_percent: 0.5
        }
    }
}

impl DrunkardsWalkArchitect {
    pub fn open_area() -> Self {
        Default::default()
    }

    pub fn open_halls() -> Self {
        Self {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 400,
            floor_percent: 0.5
        }
    }

    pub fn winding_passages() -> Self {
        Self {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 100,
            floor_percent: 0.4
        }
    }

    fn drunkard(&self, start: &Point, rng: &mut RandomNumberGenerator, mb: &mut MapBuilder) {
        let mut drunkard_pos = start.clone();
        let mut distance_staggered = 0;
        let mut tiles: Vec<usize> = Vec::new();

        loop {
            let drunk_idx = mb.map.point2d_to_index(drunkard_pos);
            tiles.push(drunk_idx);
            match rng.roll_dice(1, 4) {
                1 => drunkard_pos.x -= 1,
                2 => drunkard_pos.x += 1,
                3 => drunkard_pos.y -= 1,
                _ => drunkard_pos.y += 1
            }

            if !mb.map.in_bounds(drunkard_pos) {
                break;
            }
            // don't allow the outer border to be dug out
            if drunkard_pos.x == 0 || drunkard_pos.y == 0 {
                break;
            }
            if drunkard_pos.x as usize == MAP_WIDTH-1 || drunkard_pos.y as usize == MAP_HEIGHT-1 {
                break;
            }

            distance_staggered += 1;
            if distance_staggered > self.lifetime {
                break;
            }
        }

        if SHOW_MAPGEN_VISUALIZER {
            for tile in &tiles {
                mb.map.tiles[*tile] = TileType::DownStairs;
            }
            mb.take_snapshot();
        }
        for tile in tiles {
            mb.map.tiles[tile] = TileType::Floor;
        }
    }
}

impl MapArchitect for DrunkardsWalkArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.generate_random_table();
        mb.fill(TileType::Wall);
        mb.take_snapshot();

        let center = Point::new(MAP_WIDTH/2, MAP_HEIGHT/2);
        self.drunkard(&center, rng, &mut mb);
        mb.take_snapshot();

        let desired_floor = ((NUM_TILES as f32) * self.floor_percent) as usize;

        loop {
            while mb.map.tiles.iter().filter(|t| **t == TileType::Floor).count() < desired_floor {
                let start = if self.spawn_mode == DrunkSpawnMode::Center {
                    center
                } else {
                    Point::new(
                        rng.roll_dice(1, MAP_WIDTH as i32 - 2),
                        rng.roll_dice(1, MAP_HEIGHT as i32 - 2)
                    )
                };

                self.drunkard(
                    &start,
                    rng, &mut mb);
                mb.take_snapshot();
            }

            mb.map.populate_blocked();
            mb.prune_unreachable_regions(center);
            mb.take_snapshot();
            if mb.map.tiles.iter().filter(|t| **t == TileType::Floor).count() >= desired_floor {
                break;
            }
        }

        mb.player_start = center;
        mb.map.populate_blocked();
        mb.goal_start = mb.find_most_distant();

        mb.spawn_voronoi_regions(rng);

        mb
    }
}

