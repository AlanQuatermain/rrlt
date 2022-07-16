use crate::prelude::*;
use crate::MapTheme::Dungeon;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MapTheme {
    Dungeon,
    Forest,
}

impl Default for MapTheme {
    fn default() -> Self {
        Dungeon
    }
}

impl MapTheme {
    pub fn tile_to_render(&self, tile_type: TileType, map: &Map, idx: usize) -> FontCharType {
        match self {
            MapTheme::Dungeon => match tile_type {
                TileType::Floor => to_cp437('.'),
                TileType::Wall => self.wall_glyph(map.wall_mask(idx)),
                TileType::DownStairs => to_cp437('>'),
            },
            MapTheme::Forest => match tile_type {
                TileType::Floor => to_cp437(';'),
                TileType::Wall => to_cp437('"'),
                TileType::DownStairs => to_cp437('>'),
            },
        }
    }

    fn wall_glyph(&self, mask: u8) -> FontCharType {
        match self {
            MapTheme::Dungeon => {
                match mask {
                    0 => 9,    // Pillar (we can't see neighbors)
                    1 => 186,  // Wall only to north
                    2 => 186,  // Wall only to south
                    3 => 186,  // Walls to north and south
                    4 => 205,  // Wall only to west
                    5 => 188,  // Walls to north and west
                    6 => 187,  // Walls to south and west
                    7 => 185,  // Walls to north, south, and west
                    8 => 205,  // Wall only to east
                    9 => 200,  // Walls to north and east
                    10 => 201, // Walls to south and east
                    11 => 204, // Walls to north, south, and east
                    12 => 205, // Walls to east and west
                    13 => 202, // Walls to east, west, and north
                    14 => 203, // Walls to east, west, and south
                    15 => 206, // Wall on all sides.
                    _ => 35,   // We missed one?
                }
            }
            MapTheme::Forest => to_cp437('"'),
        }
    }
}
