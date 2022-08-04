use crate::prelude::*;

use super::{
    area_ending_points::{AreaEndingPosition, XEnd, YEnd},
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    bsp::BSPDungeonBuilder,
    cull_unreachable::CullUnreachable,
    dla::DLABuilder,
    room_corridor_spawner::CorridorSpawner,
    room_corridors_bsp::BSPCorridors,
    room_draw::RoomDrawer,
    room_sorter::{RoomSort, RoomSorter},
    voronoi_spawning::VoronoiSpawning,
};

pub fn dwarf_fort_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dwarven Fortress");
    chain.build_data.map.outdoors = false;

    chain.initial(BSPDungeonBuilder::new());
    chain.push(RoomSorter::new(RoomSort::Central));
    chain.push(RoomDrawer::new());
    chain.push(BSPCorridors::new());
    chain.push(CorridorSpawner::new());
    chain.push(DragonsLair::new());

    chain.push(AreaStartingPosition::new(XStart::Left, YStart::Top));
    chain.push(CullUnreachable::new());
    chain.push(AreaEndingPosition::new(XEnd::Right, YEnd::Bottom));
    chain.push(VoronoiSpawning::new());
    chain.push(DragonSpawner::new());

    chain
}

pub struct DragonsLair {}

impl MetaMapBuilder for DragonsLair {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DragonsLair {
    #[allow(dead_code)]
    pub fn new() -> Box<DragonsLair> {
        Box::new(DragonsLair {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.take_snapshot();

        let mut builder = BuilderChain::new(
            build_data.map.depth,
            build_data.map.width,
            build_data.map.height,
            "New Map",
        );
        builder.initial(DLABuilder::rorschach());
        builder.build_map(rng);

        // Add the history to our own.
        build_data
            .history
            .extend_from_slice(&builder.build_data.history);
        build_data.take_snapshot();

        // Merge the maps
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            if *tt == TileType::Wall && builder.build_data.map.tiles[idx] == TileType::Floor {
                *tt = TileType::Floor;
            }
        }
        build_data.take_snapshot();
    }
}

#[derive(Default)]
pub struct DragonSpawner {}

impl MetaMapBuilder for DragonSpawner {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DragonSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<DragonSpawner> {
        Box::new(Default::default())
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Find an unoccupied central location
        let seed = Point::new(build_data.map.width / 2, build_data.map.height / 2);
        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tt) in build_data.map.tiles.iter().enumerate() {
            if tt.is_walkable() {
                let pos = build_data.map.index_to_point2d(idx);
                available_floors.push((idx, DistanceAlg::PythagorasSquared.distance2d(pos, seed)));
            }
        }

        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let dragon_pt = build_data.map.index_to_point2d(available_floors[0].0);

        // Remove all spawns within 25 tiles of the drake.
        build_data.spawn_list.retain(|spawn| {
            let distance = DistanceAlg::Pythagoras.distance2d(dragon_pt, spawn.0);
            distance > 25.0
        });

        // Add the dragon
        build_data
            .spawn_list
            .push((dragon_pt, "Black Dragon".to_string()));
    }
}
