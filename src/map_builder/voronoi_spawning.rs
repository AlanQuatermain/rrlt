use std::collections::HashMap;

use crate::prelude::*;

use super::common::fill_spawns_for_region;

#[derive(Default)]
pub struct VoronoiSpawning {}

impl MetaMapBuilder for VoronoiSpawning {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl VoronoiSpawning {
    #[allow(dead_code)]
    pub fn new() -> Box<VoronoiSpawning> {
        Box::new(VoronoiSpawning::default())
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut noise_areas: HashMap<i32, Vec<Point>> = HashMap::new();
        let mut noise = FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(CellularDistanceFunction::Manhattan);

        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let pos = Point::new(x, y);
                let idx = build_data.map.point2d_to_index(pos);
                if build_data.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if noise_areas.contains_key(&cell_value) {
                        noise_areas.get_mut(&cell_value).unwrap().push(pos);
                    } else {
                        noise_areas.insert(cell_value, vec![pos]);
                    }
                }
            }
        }

        // Spawn the entries.
        for area in noise_areas.iter() {
            fill_spawns_for_region(&build_data.map, rng, area.1, &mut build_data.spawn_list);
        }
    }
}
