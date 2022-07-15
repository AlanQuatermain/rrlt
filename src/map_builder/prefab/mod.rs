use super::MapArchitect;
use crate::prelude::*;

mod levels;
mod sections;

use levels::*;
use sections::*;

pub use sections::UNDERGROUND_FORT;

#[derive(PartialEq, Clone)]
#[allow(dead_code)]
pub enum PrefabMode {
    RexLevel {
        template: &'static str,
    },
    Constant {
        level: levels::PrefabLevel,
    },
    Sectional {
        section: sections::PrefabSection,
        prior: MapBuilder,
    },
}

pub struct PrefabArchitect {
    mode: PrefabMode,
    spawns: Vec<(Point, String)>,
}

impl Default for PrefabArchitect {
    fn default() -> Self {
        Self {
            mode: PrefabMode::Constant {
                level: WFC_POPULATED,
            },
            spawns: Vec::new(),
        }
    }
}

impl MapArchitect for PrefabArchitect {
    fn new(&mut self, _rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.generate_random_table();
        mb.fill(TileType::Wall);

        match self.mode.clone() {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template, &mut mb),
            PrefabMode::Constant { level } => self.load_ascii_map(&level, &mut mb),
            PrefabMode::Sectional { section, prior } => {
                self.apply_sectional(&section, &prior, &mut mb)
            }
        }

        mb.map.populate_blocked();

        if mb.player_start == Point::zero() && mb.goal_start == Point::zero() {
            mb.player_start = mb
                .map
                .closest_floor(Point::new(MAP_WIDTH / 2, MAP_HEIGHT / 2));
            mb.goal_start = mb.find_most_distant();
            let goal_idx = mb.map.point2d_to_index(mb.goal_start);
            mb.map.tiles[goal_idx] = TileType::DownStairs;
        }

        mb.prune_unreachable_regions(mb.player_start);
        mb.take_snapshot();

        mb
    }

    fn spawn(&mut self, ecs: &mut World, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        for (pos, name) in self.spawns.iter() {
            spawn_entity(ecs, &(pos, name));
        }
    }
}

impl PrefabArchitect {
    pub fn sectional(section: PrefabSection, prior: &MapBuilder) -> PrefabArchitect {
        PrefabArchitect {
            mode: PrefabMode::Sectional {
                section,
                prior: prior.clone(),
            },
            spawns: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str, mb: &mut MapBuilder) {
        let xp_file = XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < MAP_WIDTH && y < MAP_HEIGHT {
                        let idx = map_idx(x as i32, y as i32);
                        let glyph = (cell.ch as u8) as char;
                        self.char_to_map(mb, glyph, idx);
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    fn read_ascii_to_vec(ascii: &str) -> Vec<char> {
        // Convert to a vector with newlines removed
        let mut string_vec: Vec<char> =
            ascii.chars().filter(|a| *a != '\r' && *a != '\n').collect();
        for c in string_vec.iter_mut() {
            if *c as u8 == 160u8 {
                *c = ' ';
            }
        }
        string_vec
    }

    #[allow(dead_code)]
    fn load_ascii_map(&mut self, level: &PrefabLevel, mb: &mut MapBuilder) {
        let string_vec = PrefabArchitect::read_ascii_to_vec(level.template);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < MAP_WIDTH && ty < MAP_HEIGHT {
                    let idx = map_idx(tx as i32, ty as i32);
                    self.char_to_map(mb, string_vec[i], idx);
                }
                i += 1;
            }
        }
    }

    fn apply_sectional(
        &mut self,
        section: &PrefabSection,
        prior: &MapBuilder,
        mb: &mut MapBuilder,
    ) {
        mb.history = prior.history.clone();
        mb.map = prior.map.clone();
        mb.player_start = prior.player_start;
        mb.goal_start = prior.goal_start;
        mb.take_snapshot();

        let string_vec = PrefabArchitect::read_ascii_to_vec(section.template);

        // Place the new section.
        let chunk_x = match section.placement.0 {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (MAP_WIDTH / 2) - (section.width / 2),
            HorizontalPlacement::Right => (MAP_WIDTH - 1) - section.width,
        };

        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (MAP_HEIGHT / 2) - (section.height / 2),
            VerticalPlacement::Bottom => (MAP_HEIGHT - 1) - section.height,
        };

        let region = Rect::with_size(chunk_x, chunk_y, section.width, section.height);
        mb.spawns = prior
            .spawns
            .iter()
            .filter(|pos| !region.point_in_rect(**pos))
            .map(|p| *p)
            .collect();

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < MAP_WIDTH && ty < MAP_HEIGHT {
                    let idx = mb
                        .map
                        .point2d_to_index(Point::new(tx + chunk_x, ty + chunk_y));
                    self.char_to_map(mb, string_vec[i], idx);
                }
                i += 1;
            }
        }
        mb.take_snapshot();
    }

    fn char_to_map(&mut self, mb: &mut MapBuilder, ch: char, idx: usize) {
        let pos = mb.map.index_to_point2d(idx);
        match ch {
            ' ' => mb.map.tiles[idx] = TileType::Floor,
            '#' => mb.map.tiles[idx] = TileType::Wall,
            '@' => {
                mb.map.tiles[idx] = TileType::Floor;
                mb.player_start = pos;
            }
            '>' => {
                mb.map.tiles[idx] = TileType::DownStairs;
                mb.goal_start = pos;
            }
            'g' => {
                mb.map.tiles[idx] = TileType::Floor;
                self.spawns.push((pos, "Goblin".to_string()))
            }
            'o' => {
                mb.map.tiles[idx] = TileType::Floor;
                self.spawns.push((pos, "Orc".to_string()))
            }
            'O' => {
                mb.map.tiles[idx] = TileType::Floor;
                self.spawns.push((pos, "Ogre".to_string()))
            }
            'E' => {
                mb.map.tiles[idx] = TileType::Floor;
                self.spawns.push((pos, "Ettin".to_string()))
            }
            '^' => {
                mb.map.tiles[idx] = TileType::Floor;
                self.spawns.push((pos, "Bear Trap".to_string()))
            }
            '%' => {
                mb.map.tiles[idx] = TileType::Floor;
                self.spawns.push((pos, "Rations".to_string()))
            }
            '!' => {
                mb.map.tiles[idx] = TileType::Floor;
                self.spawns.push((pos, "Health Potion".to_string()))
            }
            _ => log(format!("Unknown glyph loading map: {}", ch)),
        }
    }
}
