mod constraints;
mod solver;

use crate::prelude::*;

use constraints::*;
use solver::*;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MapChunk {
    pub pattern: Vec<TileType>,
    pub exits: [Vec<bool>; 4],
    pub has_exits: bool,
    pub compatible_with: [Vec<usize>; 4],
}

fn tile_idx_in_chunk(chunk_size: i32, x: i32, y: i32) -> usize {
    ((y * chunk_size) + x) as usize
}

#[derive(Default)]
pub struct WaveformCollapseBuilder {}

impl MetaMapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl WaveformCollapseBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<WaveformCollapseBuilder> {
        Box::new(WaveformCollapseBuilder::default())
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        const CHUNK_SIZE: i32 = 8;
        build_data.take_snapshot();

        let patterns = build_patterns(&build_data.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE, build_data);

        build_data.map = Map::new(build_data.map.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &build_data.map);
            while !solver.iteration(&mut build_data.map, rng) {
                build_data.take_snapshot();
            }
            build_data.take_snapshot();
            if solver.possible {
                break;
            }
        }
        build_data.spawn_list.clear();
    }

    fn render_tile_gallery(
        &mut self,
        constraints: &Vec<MapChunk>,
        chunk_size: i32,
        build_data: &mut BuilderMap,
    ) {
        build_data.map = Map::new(build_data.map.depth);
        let mut counter = 0;
        let mut pos = Point::new(1, 1);
        while counter < constraints.len() {
            render_pattern_to_map(&mut build_data.map, &constraints[counter], chunk_size, pos);

            pos.x += chunk_size + 1;
            if pos.x + chunk_size > build_data.map.width as i32 {
                // Move to the next row
                pos.x = 1;
                pos.y += chunk_size + 1;

                if pos.y + chunk_size > build_data.map.height as i32 {
                    // Move to the next page
                    build_data.take_snapshot();
                    build_data.map = Map::new(build_data.map.depth);

                    pos = Point::new(1, 1);
                }
            }

            counter += 1;
        }

        build_data.take_snapshot()
    }
}
