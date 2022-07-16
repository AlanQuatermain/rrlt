use crate::prelude::*;

#[derive(Default)]
pub struct DistantExit {}

impl MetaMapBuilder for DistantExit {
    fn build_map(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl DistantExit {
    #[allow(dead_code)]
    pub fn new() -> Box<DistantExit> {
        Box::new(DistantExit::default())
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let start_pos = build_data.starting_position.unwrap().clone();
        let start_idx = build_data.map.point2d_to_index(start_pos);
        build_data.map.populate_blocked();

        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = DijkstraMap::new(
            build_data.map.width,
            build_data.map.height,
            &map_starts,
            &build_data.map,
            1000.0,
        );
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in build_data.map.tiles.iter().enumerate() {
            if *tile == TileType::Floor {
                let distance = dijkstra_map.map[i];
                if distance != std::f32::MAX {
                    if distance > exit_tile.1 {
                        exit_tile = (i, distance);
                    }
                }
            }
        }

        // Place a staircase
        let stairs_idx = exit_tile.0;
        build_data.map.tiles[stairs_idx] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
