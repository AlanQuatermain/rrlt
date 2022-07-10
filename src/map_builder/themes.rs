use crate::MapTheme::Dungeon;
use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MapTheme {
    Dungeon, Forest
}

impl Default for MapTheme {
    fn default() -> Self { Dungeon }
}

impl MapTheme {
    pub fn tile_to_render(&self, tile_type: TileType) -> FontCharType {
        match self {
            MapTheme::Dungeon => { match tile_type {
                TileType::Floor => to_cp437('.'),
                TileType::Wall => to_cp437('#'),
                TileType::DownStairs => to_cp437('>'),
            }},
            MapTheme::Forest => { match tile_type {
                TileType::Floor => to_cp437(';'),
                TileType::Wall => to_cp437('"'),
                TileType::DownStairs => to_cp437('>'),
            }}
        }
    }
}