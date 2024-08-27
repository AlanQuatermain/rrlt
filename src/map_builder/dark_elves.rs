use crate::prelude::*;

use super::{
    area_ending_points::{AreaEndingPosition, XEnd, YEnd},
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    bsp_interior::BSPInteriorBuilder,
    cull_unreachable::CullUnreachable,
    voronoi_spawning::VoronoiSpawning,
};

pub fn dark_elf_city(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dark Elven City");
    chain.build_data.map.outdoors = false;

    chain.initial(BSPInteriorBuilder::new());
    chain.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.push(CullUnreachable::new());
    chain.push(AreaStartingPosition::new(XStart::Right, YStart::Center));
    chain.push(AreaEndingPosition::new(XEnd::Left, YEnd::Center));
    chain.push(VoronoiSpawning::new());

    chain
}

pub fn dark_elf_plaza(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dark Elf Plaza");
    chain.build_data.map.outdoors = false;

    chain.initial(PlazaMapBuilder::new());
    chain.push(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.push(CullUnreachable::new());

    chain
}

pub struct PlazaMapBuilder;

impl InitialMapBuilder for PlazaMapBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.empty_map(build_data);
        self.spawn_zones(rng, build_data);
    }
}

impl PlazaMapBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<PlazaMapBuilder> {
        Box::new(PlazaMapBuilder)
    }

    fn empty_map(&mut self, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Floor);
    }

    fn spawn_zones(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut voronoi_seeds: Vec<(usize, Point)> = Vec::new();
        while voronoi_seeds.len() < 32 {
            let vx = rng.roll_dice(1, build_data.map.width as i32 - 1);
            let vy = rng.roll_dice(1, build_data.map.height as i32 - 1);
            let vpos = Point::new(vx, vy);
            let vidx = build_data.map.point2d_to_index(vpos);
            let candidate = (vidx, vpos);
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance = vec![(0, 0.0f32); 32];
        let mut voronoi_membership: Vec<i32> =
            vec![0; build_data.map.width * build_data.map.height];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let ipos = build_data.map.index_to_point2d(i);
            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance = DistanceAlg::PythagorasSquared.distance2d(ipos, pos.1);
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            *vid = voronoi_distance[0].0 as i32;
        }

        // Make a list of zone sizes and cull empty ones.
        let mut zone_sizes: Vec<(i32, usize)> = Vec::with_capacity(32);
        for zone in 0..32 {
            let num_tiles = voronoi_membership.iter().filter(|z| **z == zone).count();
            if num_tiles > 0 {
                zone_sizes.push((zone, num_tiles));
            }
        }
        println!("{:?}", zone_sizes);

        zone_sizes.sort_by(|a, b| b.1.cmp(&a.1));

        // Start making zonal terrain
        zone_sizes
            .iter()
            .enumerate()
            .for_each(|(i, (zone, _))| match i {
                0 => self.portal_park(build_data, &voronoi_membership, *zone, &voronoi_seeds),
                1 | 2 => self.park(
                    build_data,
                    rng,
                    &voronoi_membership,
                    *zone,
                    &voronoi_seeds[*zone as usize].1,
                ),
                i if i > 20 => {
                    self.fill_zone(build_data, &voronoi_membership, *zone, TileType::Wall)
                }
                _ => {
                    let roll = rng.roll_dice(1, 6);
                    match roll {
                        1 => self.fill_zone(
                            build_data,
                            &voronoi_membership,
                            *zone,
                            TileType::DeepWater,
                        ),
                        2 => self.fill_zone(
                            build_data,
                            &voronoi_membership,
                            *zone,
                            TileType::ShallowWater,
                        ),
                        3 => self.stalactite_display(build_data, rng, &voronoi_membership, *zone),
                        _ => {}
                    }
                }
            });

        // Clear the path
        self.make_roads(build_data, &voronoi_membership);
    }

    fn fill_zone(
        &mut self,
        build_data: &mut BuilderMap,
        voronoi_membership: &[i32],
        zone: i32,
        tile_type: TileType,
    ) {
        voronoi_membership
            .iter()
            .enumerate()
            .filter(|(_, tile_zone)| **tile_zone == zone)
            .for_each(|(idx, _)| build_data.map.tiles[idx] = tile_type);
    }

    fn stalactite_display(
        &mut self,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        voronoi_membership: &[i32],
        zone: i32,
    ) {
        voronoi_membership
            .iter()
            .enumerate()
            .filter(|(_, tile_zone)| **tile_zone == zone)
            .for_each(|(idx, _)| {
                build_data.map.tiles[idx] = match rng.roll_dice(1, 10) {
                    1 => TileType::Stalactite,
                    2 => TileType::Stalagmite,
                    _ => TileType::Grass,
                }
            })
    }

    fn park(
        &mut self,
        build_data: &mut BuilderMap,
        rng: &mut RandomNumberGenerator,
        voronoi_membership: &[i32],
        zone: i32,
        center: &Point,
    ) {
        let zone_tiles: Vec<_> = voronoi_membership
            .iter()
            .enumerate()
            .filter(|(_, tile_zone)| **tile_zone == zone)
            .map(|(idx, _)| idx)
            .collect();

        // Start all grass
        zone_tiles
            .iter()
            .for_each(|idx| build_data.map.tiles[*idx] = TileType::Grass);

        // Add a stone area in the middle...
        for y in center.y - 2..=center.y + 2 {
            for x in center.x - 2..=center.x + 2 {
                let idx = build_data.map.point2d_to_index(Point::new(x, y));
                build_data.map.tiles[idx] = TileType::Road;
                if rng.roll_dice(1, 6) > 2 {
                    build_data.map.bloodstains.insert(idx);
                }
            }
        }

        // ...with an altar at the center...
        build_data.spawn_list.push((*center, "Altar".to_string()));

        // ...and chairs for spectators, and the spectators themselves.
        let available_enemies = match rng.roll_dice(1, 3) {
            1 => vec!["Arbat Dark Elf", "Arbat Dark Elf Leader", "Arbat Orc Slave"],
            2 => vec!["Barbo Dark Elf", "Barbo Goblin Archer"],
            _ => vec!["Cirro Dark Elf", "Cirro Priestess", "Cirro Spider"],
        };

        zone_tiles.iter().for_each(|idx| {
            if build_data.map.tiles[*idx] == TileType::Grass {
                match rng.roll_dice(1, 6) {
                    1 => build_data
                        .spawn_list
                        .push((build_data.map.index_to_point2d(*idx), "Chair".to_string())),
                    2 => {
                        let to_spawn = rng.random_slice_entry(&available_enemies).unwrap();
                        build_data
                            .spawn_list
                            .push((build_data.map.index_to_point2d(*idx), to_spawn.to_string()));
                    }
                    _ => {}
                }
            }
        })
    }

    fn make_roads(&mut self, build_data: &mut BuilderMap, voronoi_membership: &[i32]) {
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let mut neighbors = 0;
                let my_idx = build_data.map.point2d_to_index(Point::new(x, y));
                let my_seed = voronoi_membership[my_idx];

                let rect = Rect::with_size(x - 1, y - 1, 3, 3);
                for pt in rect.point_set().iter() {
                    let idx = build_data.map.point2d_to_index(*pt);
                    if idx != my_idx && voronoi_membership[idx] != my_seed {
                        neighbors += 1;
                    }
                }

                if neighbors > 1 {
                    build_data.map.tiles[my_idx] = TileType::Road;
                }
            }
        }
    }

    fn portal_park(
        &mut self,
        build_data: &mut BuilderMap,
        voronoi_membership: &[i32],
        zone: i32,
        seeds: &[(usize, Point)],
    ) {
        let zone_tiles: Vec<usize> = voronoi_membership
            .iter()
            .enumerate()
            .filter(|(_, tile_zone)| **tile_zone == zone)
            .map(|(idx, _)| idx)
            .collect();

        // Start all gravel
        zone_tiles
            .iter()
            .for_each(|idx| build_data.map.tiles[*idx] = TileType::Gravel);

        // Add the exit
        let center = seeds[zone as usize].1;
        let idx = build_data.map.point2d_to_index(center);
        build_data.map.tiles[idx] = TileType::DownStairs;

        // Add some altars around the exit
        let altars = [
            center - Point::new(2, 0),
            center + Point::new(2, 0),
            center - Point::new(0, 2),
            center + Point::new(0, 2),
        ];
        altars
            .iter()
            .for_each(|pt| build_data.spawn_list.push((*pt, "Altar".to_string())));

        build_data
            .spawn_list
            .push((center + Point::new(1, 1), "Vokoth".to_string()));
    }
}
