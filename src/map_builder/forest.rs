use crate::prelude::*;

use super::{
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    automata::CellularAutomataBuilder,
    cull_unreachable::CullUnreachable,
    distant_exit::DistantExit,
    voronoi_spawning::VoronoiSpawning,
};

#[derive(Debug, Default)]
struct YellowBrickRoad {}

pub fn forest_builder(
    new_depth: i32,
    width: usize,
    height: usize,
    _rng: &mut RandomNumberGenerator,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Woods");
    chain.build_data.map.theme = MapTheme::Forest;

    chain.initial(CellularAutomataBuilder::new());
    chain.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.push(CullUnreachable::new());
    chain.push(AreaStartingPosition::new(XStart::Left, YStart::Center));

    // Spawn mobs.
    chain.push(VoronoiSpawning::new());

    // Build a road through the forest
    chain.push(YellowBrickRoad::new());
    chain
}

impl MetaMapBuilder for YellowBrickRoad {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl YellowBrickRoad {
    #[allow(dead_code)]
    pub fn new() -> Box<YellowBrickRoad> {
        Box::new(YellowBrickRoad::default())
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.point2d_to_index(starting_pos);

        let end_pos = self.find_exit(
            build_data,
            Point::new(build_data.map.width - 2, build_data.map.height / 2),
        );
        let end_idx = build_data.map.point2d_to_index(end_pos);

        build_data.map.populate_blocked();
        let path = a_star_search(start_idx, end_idx, &mut build_data.map);
        for idx in path.steps.iter() {
            let pos = build_data.map.index_to_point2d(*idx);
            self.paint_road(build_data, pos);
            self.paint_road(build_data, pos + Point::new(-1, 0));
            self.paint_road(build_data, pos + Point::new(1, 0));
            self.paint_road(build_data, pos + Point::new(0, -1));
            self.paint_road(build_data, pos + Point::new(0, 1));
        }
        build_data.take_snapshot();

        // Place exit
        let exit_dir = rng.roll_dice(1, 2);
        let (seed, stream_start) = if exit_dir == 1 {
            (
                Point::new(build_data.map.width - 1, 1),
                Point::new(0, build_data.map.height - 1),
            )
        } else {
            (
                Point::new(build_data.map.width - 1, build_data.map.height - 1),
                Point::new(1, build_data.map.height - 1),
            )
        };

        let stairs = self.find_exit(build_data, seed);
        let stairs_idx = build_data.map.point2d_to_index(stairs);

        let stream_pos = self.find_exit(build_data, stream_start);
        let stream_idx = build_data.map.point2d_to_index(stream_pos);
        let stream = a_star_search(stairs_idx, stream_idx, &mut build_data.map);
        for tile in stream.steps.iter() {
            if build_data.map.tiles[*tile] == TileType::Floor {
                build_data.map.tiles[*tile] = TileType::ShallowWater;
            }
        }

        build_data.map.tiles[stairs_idx] = TileType::DownStairs;
    }

    fn find_exit(&self, build_data: &mut BuilderMap, seed: Point) -> Point {
        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if tiletype.is_walkable() {
                available_floors.push((
                    idx,
                    DistanceAlg::PythagorasSquared
                        .distance2d(build_data.map.index_to_point2d(idx), seed),
                ));
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        build_data.map.index_to_point2d(available_floors[0].0)
    }

    fn paint_road(&self, build_data: &mut BuilderMap, pos: Point) {
        let bounds = Rect::with_exact(1, 1, build_data.map.width - 1, build_data.map.height - 1);
        if !bounds.point_in_rect(pos) {
            return;
        }

        let idx = build_data.map.point2d_to_index(pos);
        if build_data.map.tiles[idx] != TileType::DownStairs {
            build_data.map.tiles[idx] = TileType::Road;
        }
    }
}
