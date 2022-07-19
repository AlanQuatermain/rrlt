use itertools::Itertools;
use std::collections::HashSet;

use crate::prelude::*;

use super::{
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    distant_exit::DistantExit,
};

pub fn town_builder(
    new_depth: i32,
    width: usize,
    height: usize,
    rng: &mut RandomNumberGenerator,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height);
    chain.initial(TownBuilder::new());
    chain
}

#[derive(Default, Debug)]
pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl TownBuilder {
    pub fn new() -> Box<TownBuilder> {
        Box::new(TownBuilder::default())
    }

    pub fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.grass_layer(build_data);
        self.water_and_piers(rng, build_data);

        // Make visible for screenshot
        for t in build_data.map.revealed_tiles.iter_mut() {
            *t = true;
        }
        for t in build_data.map.visible_tiles.iter_mut() {
            *t = true;
        }
        build_data.take_snapshot();

        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &doors);

        // Place the exit.
        let exit_idx = build_data
            .map
            .point2d_to_index(Point::new(build_data.map.width - 5, wall_gap_y as usize));
        build_data.map.tiles[exit_idx] = TileType::DownStairs;

        // Place the user in the largest building (the tavern).
        let building_size: Vec<(usize, i32)> = buildings
            .iter()
            .enumerate()
            .map(|(i, b)| (i, b.width() * b.height()))
            .sorted_by(|a, b| b.1.cmp(&a.1))
            .collect();

        let tavern = &buildings[building_size[0].0];
        build_data.starting_position = Some(Point::new(
            tavern.x1 + tavern.width() / 2,
            tavern.y1 + tavern.height() / 2,
        ));
    }

    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Grass);
    }

    fn water_and_piers(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535f32;
        let mut water_width: Vec<i32> = Vec::new();
        for y in 0..build_data.map.height {
            let n_water = (f32::sin(n) * 10.0) as i32 + 14 + rng.roll_dice(1, 6);
            water_width.push(n_water);
            n += 0.1;
            for x in 0..n_water {
                let idx = build_data.map.point2d_to_index(Point::new(x, y as i32));
                build_data.map.tiles[idx] = TileType::DeepWater;
            }
            for x in n_water..n_water + 3 {
                let idx = build_data.map.point2d_to_index(Point::new(x, y as i32));
                build_data.map.tiles[idx] = TileType::ShallowWater;
            }
        }
        build_data.take_snapshot();

        // Add piers
        for _ in 0..rng.roll_dice(1, 4) + 6 {
            let y = rng.roll_dice(1, build_data.map.height as i32) - 1;
            for x in 2 + rng.roll_dice(1, 6)..water_width[y as usize] + 4 {
                let idx = build_data.map.point2d_to_index(Point::new(x, y));
                build_data.map.tiles[idx] = TileType::WoodFloor;
            }
        }
        build_data.take_snapshot();
    }

    fn town_walls(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) -> (HashSet<usize>, i32) {
        let mut available_building_tiles: HashSet<usize> = HashSet::new();
        let wall_gap_y = rng.roll_dice(1, build_data.map.height as i32 - 9) + 5;

        for y in 1..build_data.map.height as i32 - 2 {
            if !(y > wall_gap_y - 4 && y < wall_gap_y + 4) {
                let idx = build_data.map.point2d_to_index(Point::new(30, y));
                build_data.map.tiles[idx] = TileType::Wall;
                build_data.map.tiles[idx - 1] = TileType::Floor;
                let idx_right = build_data
                    .map
                    .point2d_to_index(Point::new(build_data.map.width - 2, y as usize));
                build_data.map.tiles[idx_right] = TileType::Wall;
                for x in 31..build_data.map.width as i32 - 2 {
                    let gravel_idx = build_data.map.point2d_to_index(Point::new(x, y));
                    build_data.map.tiles[gravel_idx] = TileType::Gravel;
                    if y > 2 && y < build_data.map.height as i32 - 1 {
                        available_building_tiles.insert(gravel_idx);
                    }
                }
            } else {
                for x in 30..build_data.map.width as i32 {
                    let road_idx = build_data.map.point2d_to_index(Point::new(x, y));
                    build_data.map.tiles[road_idx] = TileType::Road;
                }
            }
        }
        build_data.take_snapshot();

        for x in 30..build_data.map.width - 1 {
            let idx_top = build_data.map.point2d_to_index(Point::new(x, 1));
            build_data.map.tiles[idx_top] = TileType::Wall;
            let idx_bot = build_data
                .map
                .point2d_to_index(Point::new(x, build_data.map.height - 2));
            build_data.map.tiles[idx_bot] = TileType::Wall;
        }
        build_data.take_snapshot();

        (available_building_tiles, wall_gap_y)
    }

    fn buildings(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        available_building_tiles: &mut HashSet<usize>,
    ) -> Vec<Rect> {
        let mut buildings: Vec<Rect> = Vec::new();
        let mut n_buildings = 0;

        while n_buildings < 12 {
            let rect = Rect::with_size(
                rng.roll_dice(1, build_data.map.width as i32 - 32) + 30,
                rng.roll_dice(1, build_data.map.height as i32) - 2,
                rng.roll_dice(1, 8) + 4,
                rng.roll_dice(1, 8) + 4,
            );
            let mut possible = true;
            for y in rect.y1..rect.y2 {
                for x in rect.x1..rect.x2 {
                    if x < 0
                        || x > build_data.map.width as i32 - 1
                        || y < 0
                        || y > build_data.map.height as i32 - 1
                    {
                        possible = false;
                        break;
                    } else {
                        let idx = build_data.map.point2d_to_index(Point::new(x, y));
                        if !available_building_tiles.contains(&idx) {
                            possible = false;
                            break;
                        }
                    }
                }
            }

            if possible {
                n_buildings += 1;
                buildings.push(rect);
                for y in rect.y1..rect.y2 {
                    for x in rect.x1..rect.x2 {
                        let idx = build_data.map.point2d_to_index(Point::new(x, y));
                        if x == rect.x1 || x == rect.x2 - 1 || y == rect.y1 || y == rect.y2 - 1 {
                            build_data.map.tiles[idx] = TileType::Wall;
                        } else {
                            build_data.map.tiles[idx] = TileType::WoodFloor;
                        }
                        available_building_tiles.remove(&idx);
                        available_building_tiles.remove(&(idx + 1));
                        available_building_tiles.remove(&(idx - 1));
                        available_building_tiles.remove(&(idx + build_data.map.width));
                        available_building_tiles.remove(&(idx - build_data.map.width));
                    }
                }
            }
            build_data.take_snapshot();
        }

        buildings
    }

    fn add_doors(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &mut Vec<Rect>,
        wall_gap_y: i32,
    ) -> Vec<usize> {
        let mut doors = Vec::new();
        for building in buildings.iter() {
            let door_x = building.x1 + 1 + rng.roll_dice(1, building.width() - 3);
            let cy = building.y1 + (building.height() / 2);
            let pos = if cy > wall_gap_y {
                // Door on the north wall
                Point::new(door_x, building.y1)
            } else {
                // Door on the south wall
                Point::new(door_x, building.y2 - 1)
            };
            let idx = build_data.map.point2d_to_index(pos);
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((pos, "Front Door".to_string()));
            doors.push(idx);
        }

        build_data.take_snapshot();
        doors
    }

    fn add_paths(&mut self, build_data: &mut BuilderMap, doors: &[usize]) {
        let mut roads: Vec<usize> = Vec::new();
        for (idx, tile) in build_data.map.tiles.iter().enumerate() {
            if *tile == TileType::Road {
                roads.push(idx);
            }
        }

        build_data.map.populate_blocked();
        for door_idx in doors.iter() {
            let mut nearest_roads: Vec<(usize, f32)> = Vec::new();
            let door_pt = build_data.map.index_to_point2d(*door_idx);
            for r in roads.iter() {
                let r_pt = build_data.map.index_to_point2d(*r);
                nearest_roads.push((*r, DistanceAlg::PythagorasSquared.distance2d(door_pt, r_pt)));
            }
            nearest_roads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let destination = nearest_roads[0].0;
            let path = a_star_search(*door_idx, destination, &mut build_data.map);
            if path.success {
                for step in path.steps.iter() {
                    let idx = *step as usize;
                    build_data.map.tiles[idx] = TileType::Road;
                    roads.push(idx);
                }
            }
        }
        build_data.take_snapshot();
    }
}
