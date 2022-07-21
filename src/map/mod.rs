use crate::prelude::*;
use std::collections::HashSet;

pub mod dungeon;
pub use dungeon::*;
pub mod transitions;
pub use transitions::*;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    UpStairs,
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        match self {
            TileType::Floor
            | TileType::DownStairs
            | TileType::Road
            | TileType::Grass
            | TileType::ShallowWater
            | TileType::WoodFloor
            | TileType::Bridge
            | TileType::UpStairs
            | TileType::Gravel => true,
            _ => false,
        }
    }

    pub fn is_opaque(&self) -> bool {
        match self {
            TileType::Wall => true,
            _ => false,
        }
    }

    pub fn cost(&self) -> f32 {
        match self {
            TileType::Road => 0.8,
            TileType::Grass => 1.1,
            TileType::ShallowWater => 1.2,
            _ => 1.0,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub depth: i32,
    pub name: String,
    pub theme: MapTheme,

    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub bloodstains: HashSet<usize>,
    pub view_blocked: HashSet<usize>,
    pub visible_tiles: Vec<bool>, // tiles that are always fully visible
}

impl Map {
    pub fn new<S: ToString>(depth: i32, width: usize, height: usize, name: S) -> Self {
        let num_tiles = width * height;
        Self {
            width,
            height,
            depth,
            name: name.to_string(),
            theme: MapTheme::Dungeon,
            tiles: vec![TileType::Floor; num_tiles],
            revealed_tiles: vec![false; num_tiles],
            blocked: vec![false; num_tiles],
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
            visible_tiles: vec![false; num_tiles],
        }
    }

    pub fn fill(&mut self, tile: TileType) {
        self.tiles.iter_mut().for_each(|t| *t = tile);
    }

    pub fn in_bounds(&self, point: Point) -> bool {
        point.x >= 0 && point.x < self.width as i32 && point.y >= 0 && point.y < self.height as i32
    }

    pub fn can_enter_tile(&self, point: Point) -> bool {
        self.in_bounds(point) && (self.blocked[self.idx_for_pos(&point)] == false)
    }

    pub fn try_idx(&self, point: Point) -> Option<usize> {
        if !self.in_bounds(point) {
            None
        } else {
            Some(self.idx_for_pos(&point))
        }
    }

    pub fn populate_blocked(&mut self) {
        for (idx, tile) in self.tiles.iter().enumerate() {
            self.blocked[idx] = !tile.is_walkable();
        }
    }

    fn valid_exit(&self, loc: Point, delta: Point) -> Option<usize> {
        let destination = loc + delta;
        if self.in_bounds(destination) {
            if self.can_enter_tile(destination) {
                let idx = self.idx_for_pos(&destination);
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
        let idx = self.idx_for_pos(&Point::new(x, y));
        self.revealed_tiles[idx] && self.tiles[idx] == TileType::Wall
    }

    pub fn tile_matches(&self, pos: &Point, tile: TileType) -> bool {
        let idx = self.idx_for_pos(pos);
        self.tiles[idx] == tile
    }

    fn idx_for_pos(&self, pos: &Point) -> usize {
        ((pos.y * self.width as i32) + pos.x) as usize
    }
}

impl BaseMap for Map {
    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let location = self.index_to_point2d(idx);

        for tx in -1..=1 {
            for ty in -1..=1 {
                if let Some(idx) = self.valid_exit(location, Point::new(tx, ty)) {
                    if tx == 0 || ty == 0 {
                        // Cardinal directions
                        exits.push((idx, self.tiles[idx].cost()));
                    } else {
                        // Diagonals
                        exits.push((idx, self.tiles[idx].cost() * 1.45));
                    }
                }
            }
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        DistanceAlg::Pythagoras.distance2d(self.index_to_point2d(idx1), self.index_to_point2d(idx2))
    }

    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx].is_opaque() || self.view_blocked.contains(&idx)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }

    fn in_bounds(&self, pos: Point) -> bool {
        self.in_bounds(pos)
    }

    fn point2d_to_index(&self, pt: Point) -> usize {
        self.idx_for_pos(&pt)
    }
}
