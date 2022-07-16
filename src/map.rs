use crate::prelude::*;
use std::collections::HashSet;

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 43;
pub const NUM_TILES: usize = (MAP_WIDTH * MAP_HEIGHT) as usize;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub depth: i32,

    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub bloodstains: HashSet<usize>,
}

impl Map {
    pub fn new(depth: i32) -> Self {
        Self {
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            depth,
            tiles: vec![TileType::Floor; NUM_TILES],
            revealed_tiles: vec![false; NUM_TILES],
            blocked: vec![false; NUM_TILES],
            bloodstains: HashSet::new(),
        }
    }

    pub fn fill(&mut self, tile: TileType) {
        self.tiles.iter_mut().for_each(|t| *t = tile);
    }

    pub fn in_bounds(&self, point: Point) -> bool {
        point.x >= 0 && point.x < self.width as i32 && point.y >= 0 && point.y < self.height as i32
    }

    pub fn can_enter_tile(&self, point: Point) -> bool {
        self.in_bounds(point) && (self.blocked[map_idx(point.x, point.y)] == false)
    }

    pub fn try_idx(&self, point: Point) -> Option<usize> {
        if !self.in_bounds(point) {
            None
        } else {
            Some(map_idx(point.x, point.y))
        }
    }

    pub fn populate_blocked(&mut self) {
        for (idx, tile) in self.tiles.iter().enumerate() {
            self.blocked[idx] = *tile == TileType::Wall;
        }
    }

    fn valid_exit(&self, loc: Point, delta: Point) -> Option<usize> {
        let destination = loc + delta;
        if self.in_bounds(destination) {
            if self.can_enter_tile(destination) {
                let idx = self.point2d_to_index(destination);
                Some(idx)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn wall_mask(&self, idx: usize) -> u8 {
        let pos = self.index_to_point2d(idx);
        if pos.x < 1
            || pos.x as usize > self.width - 2
            || pos.y < 1
            || pos.y as usize > self.height - 2
        {
            return 35;
        }

        let mut mask = 0;
        if self.is_revealed_and_wall(pos.x, pos.y - 1) {
            mask |= 1;
        }
        if self.is_revealed_and_wall(pos.x, pos.y + 1) {
            mask |= 2;
        }
        if self.is_revealed_and_wall(pos.x - 1, pos.y) {
            mask |= 4;
        }
        if self.is_revealed_and_wall(pos.x + 1, pos.y) {
            mask |= 8;
        }
        mask
    }

    pub fn closest_floor(&self, pos: Point) -> Point {
        let closest_point = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| **t == TileType::Floor)
            .map(|(idx, _)| {
                (
                    idx,
                    DistanceAlg::Pythagoras.distance2d(pos, self.index_to_point2d(idx)),
                )
            })
            .min_by(|(_, distance), (_, distance2)| distance.partial_cmp(&distance2).unwrap())
            .map(|(idx, _)| idx)
            .unwrap();
        self.index_to_point2d(closest_point)
    }

    fn is_revealed_and_wall(&self, x: i32, y: i32) -> bool {
        let idx = self.point2d_to_index(Point::new(x, y));
        self.revealed_tiles[idx] && self.tiles[idx] == TileType::Wall
    }
}

impl BaseMap for Map {
    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let location = self.index_to_point2d(idx);

        for tx in -1..=1 {
            for ty in -1..=1 {
                if let Some(idx) = self.valid_exit(location, Point::new(tx, ty)) {
                    exits.push((idx, 1.0));
                }
            }
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        DistanceAlg::Pythagoras.distance2d(self.index_to_point2d(idx1), self.index_to_point2d(idx2))
    }

    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }

    fn in_bounds(&self, pos: Point) -> bool {
        self.in_bounds(pos)
    }
}

pub fn map_idx(x: i32, y: i32) -> usize {
    ((y * MAP_WIDTH as i32) + x) as usize
}
