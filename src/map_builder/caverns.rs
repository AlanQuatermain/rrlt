use crate::prelude::*;

use super::{
    area_ending_points::{AreaEndingPosition, XEnd, YEnd},
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    automata::CellularAutomataBuilder,
    bsp::BSPDungeonBuilder,
    cave_decorator::CaveDecorator,
    cull_unreachable::CullUnreachable,
    distant_exit::DistantExit,
    dla::DLABuilder,
    drunkard::DrunkardsWalkBuilder,
    nearest_corridors::NearestCorridors,
    prefab::{self, PrefabBuilder},
    room_based_spawner::RoomBasedSpawner,
    room_draw::RoomDrawer,
    room_exploder::RoomExploder,
    room_sorter::{RoomSort, RoomSorter},
    voronoi_spawning::VoronoiSpawning,
};

pub fn limestone_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Limestone Caverns");
    chain.build_data.map.theme = MapTheme::LimestoneCavern;
    chain.build_data.map.outdoors = false;

    chain.initial(DrunkardsWalkBuilder::winding_passages());
    chain.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.push(CullUnreachable::new());
    chain.push(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.push(VoronoiSpawning::new());
    chain.push(DistantExit::new());
    chain.push(CaveDecorator::new());
    chain
}

pub fn limestone_deep_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Deep Limestone Caverns");
    chain.build_data.map.theme = MapTheme::LimestoneCavern;
    chain.build_data.map.outdoors = false;

    chain.initial(DLABuilder::central_attractor());
    chain.push(AreaStartingPosition::new(XStart::Left, YStart::Top));
    chain.push(VoronoiSpawning::new());
    chain.push(DistantExit::new());
    chain.push(CaveDecorator::new());
    chain.push(PrefabBuilder::sectional(prefab::sections::ORC_CAMP));
    chain
}

pub fn limestone_transition_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dwarf Fort - Upper Reaches");
    chain.build_data.map.theme = MapTheme::transition(
        MapTheme::LimestoneCavern,
        MapTheme::Dungeon,
        0.5,
        Orientation::Horizontal,
    );
    chain.build_data.map.outdoors = false;

    chain.initial(CellularAutomataBuilder::new());
    chain.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.push(CullUnreachable::new());
    chain.push(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.push(VoronoiSpawning::new());
    chain.push(CaveDecorator::new());
    chain.push(CaveTransition::new(0.5));
    chain.push(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.push(CullUnreachable::new());
    chain.push(AreaEndingPosition::new(XEnd::Right, YEnd::Center));
    chain
}

pub struct CaveTransition {
    divisor: f32,
}

impl MetaMapBuilder for CaveTransition {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveTransition {
    #[allow(dead_code)]
    pub fn new(divisor: f32) -> Box<CaveTransition> {
        Box::new(CaveTransition { divisor })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.take_snapshot();

        // Build a BSP-based dungeon
        let mut builder = BuilderChain::new(
            build_data.map.depth,
            build_data.map.width,
            build_data.map.height,
            "New Map",
        );
        builder.initial(BSPDungeonBuilder::new());
        builder.push(RoomDrawer::new());
        builder.push(RoomSorter::new(RoomSort::Rightmost));
        builder.push(NearestCorridors::new());
        builder.push(RoomExploder::new());
        builder.push(RoomBasedSpawner::new());
        builder.build_map(rng);

        // Add the history to our history
        for h in builder.build_data.history.iter() {
            build_data.history.push(h.clone());
        }
        build_data.take_snapshot();

        // Copy the right half of the BSP map into our map
        let x_border = (build_data.map.width as f32 * self.divisor) as usize;
        for x in x_border..build_data.map.width {
            for y in 0..build_data.map.height {
                let idx = build_data.map.point2d_to_index(Point::new(x, y));
                build_data.map.tiles[idx] = builder.build_data.map.tiles[idx];
            }
        }
        build_data.take_snapshot();

        // Keep Voronoi spawn data from the left side of the map
        build_data.spawn_list.retain(|s| s.0.x < x_border as i32);

        // Copy in room spawn data from the right side of the map
        for s in builder.build_data.spawn_list.iter() {
            if s.0.x >= x_border as i32 {
                build_data.spawn_list.push(s.clone());
            }
        }
    }
}
