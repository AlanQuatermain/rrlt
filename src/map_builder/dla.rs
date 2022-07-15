use super::common::*;
use super::MapArchitect;
use crate::prelude::*;

const MAP_WIDTH: i32 = super::MAP_WIDTH as i32;
const MAP_HEIGHT: i32 = super::MAP_HEIGHT as i32;

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

pub struct DLAArchitect {
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32,
}

impl Default for DLAArchitect {
    fn default() -> Self {
        Self {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        }
    }
}

impl MapArchitect for DLAArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.generate_random_table();
        mb.fill(TileType::Wall);
        mb.take_snapshot();

        mb.player_start = Point::new(MAP_WIDTH / 2, MAP_HEIGHT / 2);
        let start_idx = mb.map.point2d_to_index(mb.player_start);
        mb.map.tiles[start_idx] = TileType::Floor;
        mb.map.tiles[start_idx - 1] = TileType::Floor;
        mb.map.tiles[start_idx + 1] = TileType::Floor;
        mb.map.tiles[start_idx + super::MAP_WIDTH] = TileType::Floor;
        mb.map.tiles[start_idx - super::MAP_WIDTH] = TileType::Floor;

        let total_tiles = MAP_WIDTH * MAP_HEIGHT;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = mb
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut i = 0;
        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => self.build_walk_inwards(&mut mb.map, rng),
                DLAAlgorithm::WalkOutwards => {
                    self.build_walk_outwards(mb.player_start, &mut mb.map, rng)
                }
                DLAAlgorithm::CentralAttractor => {
                    self.build_centered(mb.player_start, &mut mb.map, rng)
                }
            }
            if i == 10 {
                mb.take_snapshot();
                i = 0;
            } else {
                i += 1;
            }
            floor_tile_count = mb
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }

        mb.map.populate_blocked();
        mb.goal_start = mb.find_most_distant();

        mb
    }

    fn spawn(&mut self, _ecs: &mut World, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        mb.spawn_voronoi_regions(rng)
    }
}

impl DLAArchitect {
    pub fn walk_inwards() -> DLAArchitect {
        DLAArchitect {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn walk_outwards() -> DLAArchitect {
        DLAArchitect {
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn central_attractor() -> DLAArchitect {
        DLAArchitect {
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn rorschach() -> DLAArchitect {
        DLAArchitect {
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::Horizontal,
            floor_percent: 0.25,
        }
    }

    fn stagger(&self, pos: Point, rng: &mut RandomNumberGenerator) -> Point {
        let stagger_direction = rng.roll_dice(1, 4);
        let mut pos = pos.clone();
        match stagger_direction {
            1 => {
                if pos.x > 2 {
                    pos.x -= 1
                }
            }
            2 => {
                if pos.x < MAP_WIDTH - 2 {
                    pos.x += 1
                }
            }
            3 => {
                if pos.y > 2 {
                    pos.y -= 1
                }
            }
            _ => {
                if pos.y < MAP_HEIGHT - 2 {
                    pos.y += 1
                }
            }
        }
        pos
    }

    fn build_walk_inwards(&self, map: &mut Map, rng: &mut RandomNumberGenerator) {
        let mut digger_pos = Point::new(
            rng.roll_dice(1, MAP_WIDTH - 3) + 1,
            rng.roll_dice(1, MAP_HEIGHT - 3) + 1,
        );
        let mut prev_pos = digger_pos.clone();

        let mut digger_idx = map.point2d_to_index(digger_pos);
        while map.tiles[digger_idx] == TileType::Wall {
            prev_pos = digger_pos.clone();
            digger_pos = self.stagger(digger_pos, rng);
            digger_idx = map.point2d_to_index(digger_pos);
        }
        paint(map, self.symmetry, self.brush_size, prev_pos);
    }

    fn build_walk_outwards(
        &mut self,
        start: Point,
        map: &mut Map,
        rng: &mut RandomNumberGenerator,
    ) {
        let mut digger_pos = start.clone();
        let mut digger_idx = map.point2d_to_index(digger_pos);
        while map.tiles[digger_idx] == TileType::Floor {
            digger_pos = self.stagger(digger_pos, rng);
            digger_idx = map.point2d_to_index(digger_pos);
        }
        paint(map, self.symmetry, self.brush_size, digger_pos);
    }

    fn build_centered(&mut self, start: Point, map: &mut Map, rng: &mut RandomNumberGenerator) {
        let mut digger_pos = Point::new(
            rng.roll_dice(1, MAP_WIDTH - 3) + 1,
            rng.roll_dice(1, MAP_HEIGHT - 3) + 1,
        );
        let mut prev_pos = digger_pos.clone();
        let mut digger_idx = map.point2d_to_index(digger_pos);

        let mut path = line2d_bresenham(digger_pos, start);

        while map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
            prev_pos = digger_pos.clone();
            digger_pos.x = path[0].x;
            digger_pos.y = path[0].y;
            path.remove(0);
            digger_idx = map.point2d_to_index(digger_pos);
        }
        paint(map, self.symmetry, self.brush_size, prev_pos);
    }
}
