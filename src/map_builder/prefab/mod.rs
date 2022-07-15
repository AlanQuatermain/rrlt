use std::collections::HashSet;

use super::MapArchitect;
use crate::prelude::*;

pub mod levels;
pub mod rooms;
pub mod sections;

use levels::*;
use rooms::*;
use sections::*;

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
    RoomVaults {
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
                level: levels::WFC_POPULATED,
            },
            spawns: Vec::new(),
        }
    }
}

impl MapArchitect for PrefabArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
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
            PrefabMode::RoomVaults { prior } => self.apply_room_vaults(depth, &prior, &mut mb, rng),
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
    pub fn rex_level(template: &'static str) -> PrefabArchitect {
        PrefabArchitect {
            mode: PrefabMode::RexLevel { template },
            spawns: Vec::new(),
        }
    }

    pub fn constant(level: levels::PrefabLevel) -> PrefabArchitect {
        PrefabArchitect {
            mode: PrefabMode::Constant { level },
            spawns: Vec::new(),
        }
    }

    pub fn sectional(section: PrefabSection, prior: &MapBuilder) -> PrefabArchitect {
        PrefabArchitect {
            mode: PrefabMode::Sectional {
                section,
                prior: prior.clone(),
            },
            spawns: Vec::new(),
        }
    }

    pub fn vaults(prior: &MapBuilder) -> PrefabArchitect {
        PrefabArchitect {
            mode: PrefabMode::RoomVaults {
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

    fn apply_prior<F>(&self, mb: &mut MapBuilder, prior: &MapBuilder, mut filter: F)
    where
        F: FnMut(Point) -> bool,
    {
        mb.history = prior.history.clone();
        mb.map = prior.map.clone();
        mb.player_start = prior.player_start;
        mb.goal_start = prior.goal_start;
        mb.take_snapshot();

        mb.spawns = prior
            .spawns
            .iter()
            .filter(|pos| filter(**pos))
            .map(|p| *p)
            .collect();
    }

    fn apply_sectional(
        &mut self,
        section: &PrefabSection,
        prior: &MapBuilder,
        mb: &mut MapBuilder,
    ) {
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
        self.apply_prior(mb, prior, |pos| !region.point_in_rect(pos));

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

    fn apply_room_vaults(
        &mut self,
        depth: i32,
        prior: &MapBuilder,
        mb: &mut MapBuilder,
        rng: &mut RandomNumberGenerator,
    ) {
        self.apply_prior(mb, prior, |_| true);

        // Do we want a vault at all?
        let vault_roll = rng.roll_dice(1, 6) + depth;
        if vault_roll < 4 {
            return;
        }

        let master_vault_list = vec![TOTALLY_NOT_A_TRAP, CHECKERBOARD, SILLY_SMILE];

        // Filter the list down to ones applicable to the current depth
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| depth >= v.first_depth && depth <= v.last_depth)
            .collect();

        if possible_vaults.is_empty() {
            return;
        }

        let n_vaults = i32::min(rng.roll_dice(1, 3), possible_vaults.len() as i32);
        let mut used_tiles: HashSet<usize> = HashSet::new();

        for _ in 0..n_vaults {
            let vault_idx = rng.random_slice_index(possible_vaults.as_slice()).unwrap();
            let vault = possible_vaults[vault_idx];

            // Make a list of places in which the vault could fit.
            let mut vault_positions: Vec<Point> = Vec::new();
            let mut idx = 0usize;
            loop {
                let pos = mb.map.index_to_point2d(idx);

                // Check that we won't overflow the map
                if pos.x > 1
                    && pos.x as usize + vault.width < (MAP_WIDTH - 2)
                    && pos.y > 1
                    && pos.y as usize + vault.height < (MAP_HEIGHT - 2)
                {
                    let mut possible = true;
                    for ty in 0..vault.height {
                        for tx in 0..vault.width {
                            let idx = mb.map.point2d_to_index(pos + Point::new(tx, ty));
                            if mb.map.tiles[idx] != TileType::Floor || used_tiles.contains(&idx) {
                                possible = false;
                                break;
                            }
                        }
                        if possible == false {
                            break;
                        }
                    }

                    if possible {
                        vault_positions.push(pos);
                    }
                }

                idx += 1;
                if idx >= mb.map.tiles.len() - 1 {
                    break;
                }
            }

            if !vault_positions.is_empty() {
                let pos_idx = rng.random_slice_index(vault_positions.as_slice()).unwrap();
                let pos = vault_positions[pos_idx];

                // clear out any spawns from our chosen area
                let region = Rect::with_size(pos.x, pos.y, vault.width as i32, vault.height as i32);
                mb.spawns.retain(|pos| !region.point_in_rect(*pos));

                let string_vec = PrefabArchitect::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for ty in 0..vault.height {
                    for tx in 0..vault.width {
                        let idx = mb.map.point2d_to_index(pos + Point::new(tx, ty));
                        self.char_to_map(mb, string_vec[i], idx);
                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                mb.take_snapshot();
                possible_vaults.remove(vault_idx);
            }
        }
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
