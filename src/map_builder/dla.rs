use crate::prelude::*;

use super::common::{paint, Symmetry};

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

pub struct DLABuilder {
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32,
}

impl Default for DLABuilder {
    fn default() -> Self {
        Self {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        }
    }
}

impl InitialMapBuilder for DLABuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Wall);
        self.build(rng, build_data);
    }
}

impl MetaMapBuilder for DLABuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DLABuilder {
    pub fn walk_inwards() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn walk_outwards() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn central_attractor() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

    pub fn rorschach() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::Horizontal,
            floor_percent: 0.25,
        })
    }

    pub fn heavy_erosion() -> Box<DLABuilder> {
        Box::new(DLABuilder {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.35,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Carve a starting seed.
        let start_pos = Point::new(build_data.map.width / 2, build_data.map.height / 2);
        let start_idx = build_data.map.point2d_to_index(start_pos);
        build_data.map.tiles[start_idx] = TileType::Floor;
        build_data.map.tiles[start_idx - 1] = TileType::Floor;
        build_data.map.tiles[start_idx + 1] = TileType::Floor;
        build_data.map.tiles[start_idx + build_data.map.width] = TileType::Floor;
        build_data.map.tiles[start_idx - build_data.map.width] = TileType::Floor;

        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut i = 0;
        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => self.build_walk_inwards(&mut build_data.map, rng),
                DLAAlgorithm::WalkOutwards => {
                    self.build_walk_outwards(start_pos, &mut build_data.map, rng)
                }
                DLAAlgorithm::CentralAttractor => {
                    self.build_centered(start_pos, &mut build_data.map, rng)
                }
            }
            if i == 10 {
                build_data.take_snapshot();
                i = 0;
            } else {
                i += 1;
            }
            floor_tile_count = build_data
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }
    }

    fn stagger(&self, map: &Map, pos: Point, rng: &mut RandomNumberGenerator) -> Point {
        let stagger_direction = rng.roll_dice(1, 4);
        let mut pos = pos.clone();
        match stagger_direction {
            1 => {
                if pos.x > 2 {
                    pos.x -= 1
                }
            }
            2 => {
                if pos.x < map.width as i32 - 2 {
                    pos.x += 1
                }
            }
            3 => {
                if pos.y > 2 {
                    pos.y -= 1
                }
            }
            _ => {
                if pos.y < map.height as i32 - 2 {
                    pos.y += 1
                }
            }
        }
        pos
    }

    fn build_walk_inwards(&self, map: &mut Map, rng: &mut RandomNumberGenerator) {
        let mut digger_pos = Point::new(
            rng.roll_dice(1, map.width as i32 - 3) + 1,
            rng.roll_dice(1, map.height as i32 - 3) + 1,
        );
        let mut prev_pos = digger_pos.clone();

        let mut digger_idx = map.point2d_to_index(digger_pos);
        while map.tiles[digger_idx] == TileType::Wall {
            prev_pos = digger_pos.clone();
            digger_pos = self.stagger(map, digger_pos, rng);
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
            digger_pos = self.stagger(map, digger_pos, rng);
            digger_idx = map.point2d_to_index(digger_pos);
        }
        paint(map, self.symmetry, self.brush_size, digger_pos);
    }

    fn build_centered(&mut self, start: Point, map: &mut Map, rng: &mut RandomNumberGenerator) {
        let mut digger_pos = Point::new(
            rng.roll_dice(1, map.width as i32 - 3) + 1,
            rng.roll_dice(1, map.height as i32 - 3) + 1,
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
