use std::collections::HashSet;

use crate::prelude::*;

pub mod levels;
pub mod rooms;
pub mod sections;

use levels::*;
use rooms::*;
use sections::*;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
    Constant { level: levels::PrefabLevel },
    Sectional { section: sections::PrefabSection },
    RoomVaults,
}

#[allow(dead_code)]
pub struct PrefabBuilder {
    mode: PrefabMode,
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_meta(rng, build_data);
    }
}

impl InitialMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_initial(rng, build_data);
    }
}

impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn rex_level(template: &'static str) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RexLevel { template },
        })
    }

    pub fn constant(level: levels::PrefabLevel) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Constant { level },
        })
    }

    pub fn sectional(section: PrefabSection) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Sectional { section },
        })
    }

    pub fn vaults() -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RoomVaults,
        })
    }

    fn build_initial(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template, build_data),
            PrefabMode::Constant { level } => self.load_ascii_map(&level, build_data),
            _ => panic!("Can't use a meta prefab as an initial builder"),
        }
    }

    fn build_meta(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        match self.mode {
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data),
            _ => panic!("Can't use an initial prefab as a meta builder"),
        }
    }

    #[allow(dead_code)]
    fn load_rex_map(&mut self, path: &str, build_data: &mut BuilderMap) {
        let xp_file = XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < build_data.map.width && y < build_data.map.height {
                        let idx = build_data.map.point2d_to_index(Point::new(x, y));
                        let glyph = (cell.ch as u8) as char;
                        self.char_to_map(glyph, idx, build_data);
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
    fn load_ascii_map(&mut self, level: &PrefabLevel, build_data: &mut BuilderMap) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < build_data.map.width && ty < build_data.map.height {
                    let idx = build_data.map.point2d_to_index(Point::new(tx, ty));
                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
    }

    fn apply_prior<F>(&self, build_data: &mut BuilderMap, mut filter: F)
    where
        F: FnMut(Point) -> bool,
    {
        build_data.spawn_list.retain(|(pos, _)| filter(*pos));
        build_data.take_snapshot();
    }

    fn apply_sectional(
        &mut self,
        section: &PrefabSection,
        _rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);

        // Place the new section.
        let chunk_x = match section.placement.0 {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (build_data.map.width / 2) - (section.width / 2),
            HorizontalPlacement::Right => (build_data.map.width - 1) - section.width,
        };

        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (build_data.map.height / 2) - (section.height / 2),
            VerticalPlacement::Bottom => (build_data.map.height - 1) - section.height,
        };

        let region = Rect::with_size(chunk_x, chunk_y, section.width, section.height);
        self.apply_prior(build_data, |pos| !region.point_in_rect(pos));

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx < build_data.map.width && ty < build_data.map.height {
                    let idx = build_data
                        .map
                        .point2d_to_index(Point::new(tx + chunk_x, ty + chunk_y));
                    self.char_to_map(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }

    fn apply_room_vaults(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.apply_prior(build_data, |_| true);

        // Do we want a vault at all?
        let vault_roll = rng.roll_dice(1, 6) + build_data.map.depth;
        if vault_roll < 4 {
            return;
        }

        let master_vault_list = vec![TOTALLY_NOT_A_TRAP, CHECKERBOARD, SILLY_SMILE];

        // Filter the list down to ones applicable to the current depth
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| {
                build_data.map.depth >= v.first_depth && build_data.map.depth <= v.last_depth
            })
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
                let pos = build_data.map.index_to_point2d(idx);

                // Check that we won't overflow the map
                if pos.x > 1
                    && pos.x as usize + vault.width < (build_data.map.width - 2)
                    && pos.y > 1
                    && pos.y as usize + vault.height < (build_data.map.height - 2)
                {
                    let mut possible = true;
                    for ty in 0..vault.height {
                        for tx in 0..vault.width {
                            let idx = build_data.map.point2d_to_index(pos + Point::new(tx, ty));
                            if build_data.map.tiles[idx] != TileType::Floor
                                || used_tiles.contains(&idx)
                            {
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
                if idx >= build_data.map.tiles.len() - 1 {
                    break;
                }
            }

            if !vault_positions.is_empty() {
                let pos_idx = rng.random_slice_index(vault_positions.as_slice()).unwrap();
                let pos = vault_positions[pos_idx];

                // clear out any spawns from our chosen area
                let region = Rect::with_size(pos.x, pos.y, vault.width as i32, vault.height as i32);
                build_data
                    .spawn_list
                    .retain(|pos| !region.point_in_rect(pos.0));

                let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for ty in 0..vault.height {
                    for tx in 0..vault.width {
                        let idx = build_data.map.point2d_to_index(pos + Point::new(tx, ty));
                        self.char_to_map(string_vec[i], idx, build_data);
                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                build_data.take_snapshot();
                possible_vaults.remove(vault_idx);
            }
        }
    }

    fn char_to_map(&mut self, ch: char, idx: usize, build_data: &mut BuilderMap) {
        let pos = build_data.map.index_to_point2d(idx);
        match ch {
            ' ' => build_data.map.tiles[idx] = TileType::Floor,
            '#' => build_data.map.tiles[idx] = TileType::Wall,
            '@' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.starting_position = Some(pos);
            }
            '>' => {
                build_data.map.tiles[idx] = TileType::DownStairs;
            }
            'e' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((pos, "Dark Elf".to_string()))
            }
            'k' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((pos, "Kobold".to_string()))
            }
            'g' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((pos, "Goblin".to_string()))
            }
            'o' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((pos, "Orc".to_string()))
            }
            'O' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((pos, "Orc Leader".to_string()))
            }
            '^' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((pos, "Bear Trap".to_string()))
            }
            '%' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((pos, "Rations".to_string()))
            }
            '!' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data
                    .spawn_list
                    .push((pos, "Health Potion".to_string()))
            }
            '≈' => {
                build_data.map.tiles[idx] = TileType::DeepWater;
            }
            '☼' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((pos, "Watch Fire".to_string()))
            }
            _ => log(format!("Unknown glyph loading map: {}", ch)),
        }
    }
}
