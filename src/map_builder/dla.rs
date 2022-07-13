use crate::prelude::*;
use super::MapArchitect;

const MAP_WIDTH: i32 = super::MAP_WIDTH as i32;
const MAP_HEIGHT: i32 = super::MAP_HEIGHT as i32;

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm { WalkInwards, WalkOutwards, CentralAttractor }

#[derive(PartialEq, Copy, Clone)]
pub enum DLASymmetry { None, Horizontal, Vertical, Both }

pub struct DLAArchitect {
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: DLASymmetry,
    floor_percent: f32
}

impl Default for DLAArchitect {
    fn default() -> Self {
        Self {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25
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

        mb.player_start = Point::new(MAP_WIDTH/2, MAP_HEIGHT/2);
        let start_idx = mb.map.point2d_to_index(mb.player_start);
        mb.map.tiles[start_idx] = TileType::Floor;
        mb.map.tiles[start_idx-1] = TileType::Floor;
        mb.map.tiles[start_idx+1] = TileType::Floor;
        mb.map.tiles[start_idx+super::MAP_WIDTH] = TileType::Floor;
        mb.map.tiles[start_idx-super::MAP_WIDTH] = TileType::Floor;

        let total_tiles = MAP_WIDTH * MAP_HEIGHT;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = mb.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        let mut i = 0;
        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => self.build_walk_inwards(&mut mb.map, rng),
                DLAAlgorithm::WalkOutwards => self.build_walk_outwards(mb.player_start, &mut mb.map, rng),
                DLAAlgorithm::CentralAttractor => self.build_centered(mb.player_start,&mut mb.map, rng),
            }
            if i == 10 {
                mb.take_snapshot();
                i = 0;
            }
            else {
                i += 1;
            }
            floor_tile_count = mb.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }

        mb.map.populate_blocked();
        mb.goal_start = mb.find_most_distant();
        mb.spawn_voronoi_regions(rng);

        mb
    }
}

impl DLAArchitect {
    pub fn walk_inwards() -> DLAArchitect {
        DLAArchitect {
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25
        }
    }

    pub fn walk_outwards() -> DLAArchitect {
        DLAArchitect {
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25
        }
    }

    pub fn central_attractor() -> DLAArchitect {
        DLAArchitect {
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25
        }
    }

    pub fn rorschach() -> DLAArchitect {
        DLAArchitect {
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: DLASymmetry::Horizontal,
            floor_percent: 0.25
        }
    }

    fn stagger(&self, x: &mut i32, y: &mut i32, rng: &mut RandomNumberGenerator) {
        let stagger_direction = rng.roll_dice(1, 4);
        match stagger_direction {
            1 => if *x > 2 { *x -= 1 },
            2 => if *x < MAP_WIDTH-2 { *x += 1 },
            3 => if *y > 2 { *y -= 1 },
            _ => if *y < MAP_HEIGHT-2 { *y += 1 },
        }
    }

    fn build_walk_inwards(&self, map: &mut Map, rng: &mut RandomNumberGenerator) {
        let mut digger_x = rng.roll_dice(1, MAP_WIDTH-3) + 1;
        let mut digger_y = rng.roll_dice(1, MAP_HEIGHT-3) + 1;
        let mut prev_x = digger_x;
        let mut prev_y = digger_y;

        let mut digger_idx = map_idx(digger_x, digger_y);
        while map.tiles[digger_idx] == TileType::Wall {
            prev_x = digger_x;
            prev_y = digger_y;
            self.stagger(&mut digger_x, &mut digger_y, rng);
            digger_idx = map_idx(digger_x, digger_y);
        }
        self.paint(prev_x, prev_y, map);
    }

    fn build_walk_outwards(&mut self, start: Point, map: &mut Map, rng: &mut RandomNumberGenerator) {
        let mut digger_x = start.x;
        let mut digger_y = start.y;
        let mut digger_idx = map_idx(digger_x, digger_y);
        while map.tiles[digger_idx] == TileType::Floor {
            self.stagger(&mut digger_x, &mut digger_y, rng);
            digger_idx = map_idx(digger_x, digger_y);
        }
        self.paint(digger_x, digger_y, map);
    }

    fn build_centered(&mut self, start: Point, map: &mut Map, rng: &mut RandomNumberGenerator) {
        let mut digger_x = rng.roll_dice(1, MAP_WIDTH - 3) + 1;
        let mut digger_y = rng.roll_dice(1, MAP_HEIGHT - 3) + 1;
        let mut prev_x = digger_x;
        let mut prev_y = digger_y;
        let mut digger_idx = map_idx(digger_x, digger_y);

        let mut path = line2d_bresenham(Point::new(digger_x, digger_y), start);

        while map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
            prev_x = digger_x;
            prev_y = digger_y;
            digger_x = path[0].x;
            digger_y = path[0].y;
            path.remove(0);
            digger_idx = map_idx(digger_x, digger_y);
        }
        self.paint(prev_x, prev_y, map);
    }

    fn paint(&self, x: i32, y: i32, map: &mut Map) {
        match self.symmetry {
            DLASymmetry::None => self.apply_paint(x, y, map),
            DLASymmetry::Horizontal => {
                let center_x = MAP_WIDTH / 2;
                if x == center_x {
                    self.apply_paint(x, y, map);
                }
                else {
                    let dist_x = i32::abs(center_x - x);
                    self.apply_paint(center_x + dist_x, y, map);
                    self.apply_paint(center_x - dist_x, y, map);
                }
            }
            DLASymmetry::Vertical => {
                let center_y = MAP_HEIGHT / 2;
                if y == center_y {
                    self.apply_paint(x, y, map);
                }
                else {
                    let dist_y = i32::abs(center_y - y);
                    self.apply_paint(x, center_y + dist_y, map);
                    self.apply_paint(x, center_y - dist_y, map);
                }
            }
            DLASymmetry::Both => {
                let center_x = MAP_WIDTH / 2;
                let center_y = MAP_HEIGHT / 2;
                if x == center_x && y == center_y {
                    self.apply_paint(x, y, map);
                }
                else {
                    let dist_x = i32::abs(center_x - x);
                    self.apply_paint(center_x + dist_x, y, map);
                    self.apply_paint(center_x - dist_x, y, map);
                    let dist_y = i32::abs(center_y - y);
                    self.apply_paint(x, center_y + dist_y, map);
                    self.apply_paint(x, center_y - dist_y, map);
                }
            }
        }
    }

    fn apply_paint(&self, x: i32, y: i32, map: &mut Map) {
        match self.brush_size {
            1 => {
                let digger_idx = map_idx(x, y);
                map.tiles[digger_idx] = TileType::Floor;
            }
            _ => {
                let half_brush_size = self.brush_size/2;
                for brush_y in y-half_brush_size .. y+half_brush_size {
                    for brush_x in x-half_brush_size .. x+half_brush_size {
                        if brush_x > 1 && brush_x < MAP_WIDTH-1 && brush_y > 1 && brush_y < MAP_HEIGHT-1 {
                            let idx = map_idx(brush_x, brush_y);
                            map.tiles[idx] = TileType::Floor;
                        }
                    }
                }
            }
        }
    }
}