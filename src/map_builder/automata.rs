use std::collections::HashMap;
use crate::prelude::*;
use super::MapArchitect;
use itertools::Itertools;
use crate::map_builder::MAX_SPAWNS_PER_ROOM;

#[derive(Default)]
pub struct CellularAutomataArchitect {
    noise_areas: HashMap<i32, Vec<usize>>
}

impl CellularAutomataArchitect {
    fn build_noise_areas(&mut self, rng: &mut RandomNumberGenerator, map: &Map) {
        let mut noise  = FastNoise::seeded(rng.next_u64());
        noise.set_noise_type(NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(CellularDistanceFunction::Manhattan);

        for y in 1 .. MAP_HEIGHT as i32 - 1 {
            for x in 1 .. MAP_WIDTH as i32 - 1 {
                let idx = map_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if self.noise_areas.contains_key(&cell_value) {
                        self.noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    }
                    else {
                        self.noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }
    }

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
        let center = Point::new(MAP_WIDTH/2, MAP_HEIGHT/2);
        let closest_point = map.tiles.iter().enumerate()
            .filter(|(_, t)| **t == TileType::Floor)
            .map(|(idx, _)| (idx, DistanceAlg::Pythagoras.distance2d(
                center, map.index_to_point2d(idx))))
            .min_by(|(_, distance), (_, distance2)| {
                distance.partial_cmp(&distance2).unwrap()
            })
            .map(|(idx, _)| idx)
            .unwrap();
        map.index_to_point2d(closest_point)
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

        self.build_noise_areas(rng, &mb.map);
        mb.map.populate_blocked();

        let start = self.find_start(&mb.map);
        mb.player_start = start;
        mb.goal_start = mb.find_most_distant();
        println!("Goal start: {:?}", mb.goal_start);

        for area in self.noise_areas.iter() {
            let num_spawns = i32::min(area.1.len() as i32, rng.roll_dice(1, MAX_SPAWNS_PER_ROOM + 3) + (depth) - 3);
            if num_spawns == 0 { continue; }

            let mut values = area.1.clone();
            for _ in 0 .. num_spawns {
                let idx = rng.random_slice_index(values.as_slice()).unwrap();
                mb.spawns.push(mb.map.index_to_point2d(values[idx]));
                values.remove(idx);
            }
        }

        mb
    }
}