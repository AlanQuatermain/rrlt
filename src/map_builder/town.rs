use std::collections::HashSet;

use crate::prelude::*;

pub fn town_builder(
    new_depth: i32,
    width: usize,
    height: usize,
    _rng: &mut RandomNumberGenerator,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "The Town of Brackleton");
    chain.initial(TownBuilder::new());
    chain
}

#[derive(Copy, Clone, Debug)]
enum BuildingTag {
    Pub,
    Temple,
    Blacksmith,
    Clothier,
    Alchemist,
    PlayerHouse,
    Hovel,
    Abandoned,
    Unassigned,
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
        // for t in build_data.map.revealed_tiles.iter_mut() {
        //     *t = true;
        // }
        // for t in build_data.map.visible_tiles.iter_mut() {
        //     *t = true;
        // }
        build_data.take_snapshot();

        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &doors);

        // Place the exit.
        for y in wall_gap_y - 3..wall_gap_y + 4 {
            let exit_idx = build_data
                .map
                .point2d_to_index(Point::new(build_data.map.width - 2, y as usize));
            build_data.map.tiles[exit_idx] = TileType::DownStairs;
        }

        let building_sizes = self.sort_buildings(&buildings);
        self.building_factory(rng, build_data, &buildings, &building_sizes);

        self.spawn_dockers(build_data, rng);
        self.spawn_townsfolk(build_data, rng, &mut available_building_tiles);

        // We know what our own hometown looks like, thanks.
        build_data
            .map
            .revealed_tiles
            .iter_mut()
            .for_each(|t| *t = true);
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
                build_data.map.tiles[idx] = TileType::Bridge;
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
                build_data.take_snapshot();
            }
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
            build_data.spawn_list.push((pos, "Door".to_string()));
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

    fn sort_buildings(&mut self, buildings: &[Rect]) -> Vec<(usize, i32, BuildingTag)> {
        let mut building_size: Vec<(usize, i32, BuildingTag)> = Vec::new();
        for (i, building) in buildings.iter().enumerate() {
            building_size.push((
                i,
                building.width() * building.height(),
                BuildingTag::Unassigned,
            ));
        }
        building_size.sort_by(|a, b| b.1.cmp(&a.1));

        building_size[0].2 = BuildingTag::Pub;
        building_size[1].2 = BuildingTag::Temple;
        building_size[2].2 = BuildingTag::Blacksmith;
        building_size[3].2 = BuildingTag::Clothier;
        building_size[4].2 = BuildingTag::Alchemist;
        building_size[5].2 = BuildingTag::PlayerHouse;
        for b in building_size.iter_mut().skip(6) {
            b.2 = BuildingTag::Hovel;
        }
        building_size.last_mut().unwrap().2 = BuildingTag::Abandoned;

        building_size
    }

    fn building_factory(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &[Rect],
        building_index: &[(usize, i32, BuildingTag)],
    ) {
        for (idx, area, build_type) in building_index.iter() {
            let building = &buildings[*idx];
            match build_type {
                BuildingTag::Pub => self.build_pub(building, build_data, rng),
                BuildingTag::Temple => self.build_temple(building, build_data, rng),
                BuildingTag::Blacksmith => self.build_smith(building, build_data, rng),
                BuildingTag::Clothier => self.build_clothier(building, build_data, rng),
                BuildingTag::Alchemist => self.build_alchemist(building, build_data, rng),
                BuildingTag::PlayerHouse => self.build_my_house(building, build_data, rng),
                BuildingTag::Hovel => self.build_hovel(building, build_data, rng),
                BuildingTag::Abandoned => self.build_abandoned_house(building, build_data, rng),
                _ => {}
            }
        }
    }

    fn random_building_spawn(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        to_place: &mut Vec<&str>,
        player_idx: usize,
    ) {
        for y in building.y1 + 1..building.y2 {
            for x in building.x1 + 1..building.x2 {
                let build_pos = Point::new(x, y);
                let idx = build_data.map.point2d_to_index(build_pos);
                if build_data.map.tiles[idx].is_walkable()
                    && idx != player_idx
                    && rng.roll_dice(1, 3) == 1
                    && !to_place.is_empty()
                {
                    let entity_tag = to_place[0];
                    to_place.remove(0);
                    build_data
                        .spawn_list
                        .push((build_pos, entity_tag.to_string()));
                }
            }
        }
    }

    fn build_pub(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place the player
        let player_pos = Point::new(
            building.x1 + (building.width() / 2),
            building.y1 + (building.height() / 2),
        );
        build_data.starting_position = Some(player_pos);
        let player_idx = build_data.map.point2d_to_index(player_pos);

        // Place other items
        let mut to_place = vec![
            "Barkeep",
            "Shady Salesman",
            "Patron",
            "Patron",
            "Keg",
            "Table",
            "Chair",
            "Table",
            "Chair",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, player_idx);
    }

    fn build_temple(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place: Vec<&str> = vec![
            "Priest",
            "Altar",
            "Parishioner",
            "Parishioner",
            "Chair",
            "Chair",
            "Candle",
            "Candle",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_smith(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place: Vec<&str> = vec![
            "Blacksmith",
            "Anvil",
            "Water Trough",
            "Weapon Rack",
            "Armor Stand",
        ];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_clothier(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place: Vec<&str> = vec!["Clothier", "Cabinet", "Table", "Loom", "Hide Rack"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_alchemist(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place: Vec<&str> =
            vec!["Alchemist", "Chemistry Set", "Dead Thing", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_my_house(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place: Vec<&str> = vec!["Mom", "Bed", "Cabinet", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_hovel(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Place items
        let mut to_place: Vec<&str> = vec!["Peasant", "Bed", "Chair", "Table"];
        self.random_building_spawn(building, build_data, rng, &mut to_place, 0);
    }

    fn build_abandoned_house(
        &mut self,
        building: &Rect,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
    ) {
        // Rats!
        for y in building.y1 + 1..building.y2 {
            for x in building.x1 + 1..building.x2 {
                let build_pos = Point::new(x, y);
                let idx = build_data.map.point2d_to_index(build_pos);
                if build_data.map.tiles[idx].is_walkable() && rng.roll_dice(1, 2) == 1 {
                    build_data.spawn_list.push((build_pos, "Rat".to_string()));
                }
            }
        }
    }

    fn spawn_dockers(&mut self, build_data: &mut BuilderMap, rng: &mut RandomNumberGenerator) {
        for (idx, tt) in build_data.map.tiles.iter().enumerate() {
            if *tt == TileType::Bridge && rng.roll_dice(1, 6) == 1 {
                let roll = rng.roll_dice(1, 3);
                let pos = build_data.map.index_to_point2d(idx);
                match roll {
                    1 => build_data.spawn_list.push((pos, "Dock Worker".to_string())),
                    2 => build_data
                        .spawn_list
                        .push((pos, "Wannabe Pirate".to_string())),
                    _ => build_data.spawn_list.push((pos, "Fisher".to_string())),
                }
            }
        }
    }

    fn spawn_townsfolk(
        &mut self,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        available_building_tiles: &mut HashSet<usize>,
    ) {
        for idx in available_building_tiles.iter() {
            if rng.roll_dice(1, 14) == 1 {
                let pos = build_data.map.index_to_point2d(*idx);
                match rng.roll_dice(1, 4) {
                    1 => build_data.spawn_list.push((pos, "Peasant".to_string())),
                    2 => build_data.spawn_list.push((pos, "Drunk".to_string())),
                    3 => build_data.spawn_list.push((pos, "Dock Worker".to_string())),
                    _ => build_data.spawn_list.push((pos, "Fisher".to_string())),
                }
            }
        }
    }
}
