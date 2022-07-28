use crate::prelude::*;
use crate::MapTheme::Dungeon;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MapTheme {
    Dungeon,
    Forest,
    LimestoneCavern,
}

impl Default for MapTheme {
    fn default() -> Self {
        Dungeon
    }
}

impl MapTheme {
    pub fn default_glyph_for_tile(&self, map: &Map, idx: usize) -> (FontCharType, RGB) {
        match map.tiles[idx] {
            TileType::Floor => (to_cp437('.'), RGB::named(TEAL)),
            TileType::WoodFloor => (to_cp437('.'), RGB::named(CHOCOLATE1)),
            TileType::Wall => (self.wall_glyph(map.wall_mask(idx)), RGB::named(GREEN)),
            TileType::DownStairs => (to_cp437('>'), RGB::named(CYAN)),
            TileType::Bridge => (to_cp437('░'), RGB::named(CHOCOLATE)),
            TileType::Road => (to_cp437('≡'), RGB::named(DIMGREY)),
            TileType::Grass => (to_cp437('"'), RGB::named(LIGHTGREEN)),
            TileType::ShallowWater => (to_cp437('~'), RGB::named(CYAN)),
            TileType::DeepWater => (to_cp437('~'), RGB::named(BLUE)),
            TileType::Gravel => (to_cp437(';'), RGB::named(LIGHTSLATEGREY)),
            TileType::UpStairs => (to_cp437('<'), RGB::named(CYAN)),
            TileType::Stalactite => (to_cp437('╨'), RGB::named(GREY50)),
            TileType::Stalagmite => (to_cp437('╥'), RGB::named(GREY50)),
        }
    }

    pub fn tile_to_render(&self, map: &Map, idx: usize) -> (FontCharType, RGB) {
        match self {
            MapTheme::Dungeon => self.default_glyph_for_tile(map, idx),
            MapTheme::Forest => match map.tiles[idx] {
                TileType::Wall => (to_cp437('♣'), RGB::from_f32(0., 0.6, 0.)),
                TileType::Road => (to_cp437('≡'), RGB::named(YELLOW)),
                TileType::Floor | TileType::WoodFloor => (to_cp437('"'), RGB::named(LIGHTGREEN)),
                _ => self.default_glyph_for_tile(map, idx),
            },
            MapTheme::LimestoneCavern => match map.tiles[idx] {
                TileType::Wall => (to_cp437('▒'), RGB::from_f32(0.7, 0.7, 0.7)),
                TileType::Bridge => (to_cp437('.'), RGB::named(CHOCOLATE)),
                TileType::Road => (to_cp437('≡'), RGB::named(YELLOW)),
                TileType::ShallowWater => (to_cp437('░'), RGB::named(CYAN)),
                TileType::DeepWater => (to_cp437('▓'), RGB::from_f32(0.2, 0.2, 1.0)),
                TileType::Floor => (to_cp437('░'), RGB::from_f32(0.4, 0.4, 0.4)),
                TileType::WoodFloor => (to_cp437('░'), RGB::named(CHOCOLATE2)),
                _ => self.default_glyph_for_tile(map, idx),
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
            MapTheme::LimestoneCavern => to_cp437('▒'),
        }
    }
}
