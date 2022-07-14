mod image_loader;
mod constraints;
mod solver;

use crate::prelude::*;
use super::MapArchitect;

use constraints::*;
use image_loader::*;
use solver::*;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct MapChunk {
    pub pattern: Vec<TileType>,
    pub exits: [Vec<bool>; 4],
    pub has_exits: bool,
    pub compatible_with: [Vec<usize>; 4]
}

fn tile_idx_in_chunk(chunk_size: i32, x: i32, y: i32) -> usize {
    ((y * chunk_size) + x) as usize
}

#[derive(PartialEq, Copy, Clone)]
pub enum WaveformMode { TestMap, Derived }

pub struct WaveformCollapseArchitect {
    mode: WaveformMode,
    derive_from: Option<Box<dyn MapArchitect>>
}

const CHUNK_SIZE: i32 = 8;

impl Default for WaveformCollapseArchitect {
    fn default() -> Self {
        WaveformCollapseArchitect {
            mode: WaveformMode::TestMap,
            derive_from: None
        }
    }
}

impl MapArchitect for WaveformCollapseArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.generate_random_table();

        if self.mode == WaveformMode::TestMap {
            mb.map = load_rex_map(
                &XpFile::from_resource("../resources/wfc-demo1.xp").unwrap());
            mb.take_snapshot();

            mb.player_start = mb.map.closest_floor(Point::new(MAP_WIDTH/2, MAP_HEIGHT/2));

            mb.map.populate_blocked();
            mb.prune_unreachable_regions(mb.player_start);
            mb.take_snapshot();
        }
        else {
            if let Some(mut src_map) = self.derive_from.as_mut().map(|a| a.new(rng, depth).map.clone()) {
                for t in src_map.tiles.iter_mut() {
                    if *t == TileType::DownStairs { *t = TileType::Floor; }
                }
                mb.map = src_map;
            } else {
                mb.map = super::CellularAutomataArchitect::default().new(rng, depth).map
            }

            let patterns = build_patterns(&mb.map, CHUNK_SIZE, true, true);
            let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
            let desired_floor = (0.4f32 * (MAP_WIDTH * MAP_HEIGHT) as f32) as usize;

            loop {
                mb.map = Map::new();
                mb.history.clear();
                self.render_tile_gallery(&mut mb, &constraints, CHUNK_SIZE);
                mb.take_snapshot();

                mb.map = Map::new();
                mb.fill(TileType::Wall);

                loop {
                    let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &mb.map);
                    while !solver.iteration(&mut mb.map, rng) {
                        mb.take_snapshot();
                    }
                    mb.take_snapshot();
                    if solver.possible { break; }
                }

                mb.player_start = mb.map.closest_floor(Point::new(MAP_WIDTH / 2, MAP_HEIGHT / 2));

                mb.map.populate_blocked();
                mb.prune_unreachable_regions(mb.player_start);

                if mb.map.tiles.iter().filter(|t| **t == TileType::Floor).count() >= desired_floor {
                    break;
                }
            }
        }

        mb.goal_start = mb.find_most_distant();
        mb.take_snapshot();

        mb.spawn_voronoi_regions(rng);

        mb
    }
}

impl WaveformCollapseArchitect {
    pub fn new(mode: WaveformMode, derive_from: Option<Box<dyn MapArchitect>>) -> Self {
        WaveformCollapseArchitect {
            mode, derive_from
        }
    }

    #[allow(dead_code)]
    pub fn test_map() -> WaveformCollapseArchitect {
        WaveformCollapseArchitect::new(WaveformMode::TestMap, None)
    }

    pub fn derived_map(architect: Box<dyn MapArchitect>) -> WaveformCollapseArchitect {
        WaveformCollapseArchitect::new(WaveformMode::Derived, Some(architect))
    }

    fn render_tile_gallery(
        &mut self,
        mb: &mut MapBuilder,
        constraints: &Vec<MapChunk>,
        chunk_size: i32
    ) {
        mb.map = Map::new();
        let mut counter = 0;
        let mut pos = Point::new(1, 1);
        while counter < constraints.len() {
            render_pattern_to_map(&mut mb.map, &constraints[counter], chunk_size, pos);

            pos.x += chunk_size + 1;
            if pos.x + chunk_size > MAP_WIDTH as i32 {
                // Move to the next row
                pos.x = 1;
                pos.y += chunk_size + 1;

                if pos.y + chunk_size > MAP_HEIGHT as i32 {
                    // Move to the next page
                    mb.take_snapshot();
                    mb.map = Map::new();

                    pos = Point::new(1, 1);
                }
            }

            counter += 1;
        }

        mb.take_snapshot()
    }
}