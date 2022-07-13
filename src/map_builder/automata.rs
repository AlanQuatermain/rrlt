use std::collections::HashMap;
use crate::prelude::*;
use super::MapArchitect;
use itertools::Itertools;
use super::MAX_SPAWNS_PER_ROOM;

#[derive(Default)]
pub struct CellularAutomataArchitect {}

impl CellularAutomataArchitect {
    fn random_noise_map(&self, rng: &mut RandomNumberGenerator, map: &mut Map) {
        for y in 1 .. MAP_HEIGHT as i32 - 1 {
            for x in 1 .. MAP_WIDTH as i32 - 1 {
                let roll = rng.roll_dice(1, 100);
                if roll > 55 {
                    map.tiles[map_idx(x, y)] = TileType::Floor;
                }
                else {
                    map.tiles[map_idx(x, y)] = TileType::Wall;
                }
            }
        }
    }

    fn count_neighbors(&self, x: i32, y: i32, map: &Map) -> usize {
        let mut neighbors = 0;
        for iy in -1..=1 {
            for ix in -1..=1 {
                if !(ix == 0 && iy == 0) && map.tiles[map_idx(x+ix, y+iy)] == TileType::Wall {
                    neighbors += 1;
                }
            }
        }
        neighbors
    }

    fn iteration(&self, map: &mut Map) {
        let mut new_tiles = map.tiles.clone();
        for y in 1 .. MAP_HEIGHT as i32 - 1 {
            for x in 1 .. MAP_WIDTH as i32 - 1 {
                let neighbors = self.count_neighbors(x, y, map);
                let idx = map_idx(x, y);
                if neighbors > 4 || neighbors == 0 {
                    new_tiles[idx] = TileType::Wall;
                }
                else {
                    new_tiles[idx] = TileType::Floor;
                }
            }
        }
        map.tiles = new_tiles;
    }

    fn find_start(&self, map: &Map) -> Point {
        map.closest_floor(Point::new(MAP_WIDTH/2, MAP_HEIGHT/2))
    }
}

impl MapArchitect for CellularAutomataArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.generate_random_table();
        mb.fill(TileType::Wall);

        self.random_noise_map(rng, &mut mb.map);
        mb.take_snapshot();
        for _ in 0 .. 10 {
            self.iteration(&mut mb.map);
            mb.take_snapshot();
        }

        let start = self.find_start(&mb.map);
        mb.player_start = start;

        mb.prune_unreachable_regions(start);
        mb.take_snapshot();
        mb.map.populate_blocked();
        mb.goal_start = mb.find_most_distant();
        mb.spawn_voronoi_regions(rng);

        mb
    }
}