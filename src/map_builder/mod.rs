use crate::prelude::*;

use self::{
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    automata::CellularAutomataBuilder,
    bsp::BSPDungeonBuilder,
    bsp_interior::BSPInteriorBuilder,
    cull_unreachable::CullUnreachable,
    distant_exit::DistantExit,
    dla::DLABuilder,
    drunkard::DrunkardsWalkBuilder,
    maze::MazeBuilder,
    prefab::PrefabBuilder,
    room_based_spawner::RoomBasedSpawner,
    room_based_stairs::RoomBasedStairs,
    room_based_starting_position::RoomBasedStartingPosition,
    room_corner_rounding::RoomCornerRounder,
    room_corridors_bsp::BSPCorridors,
    room_corridors_dogleg::DoglegCorridors,
    room_draw::RoomDrawer,
    room_exploder::RoomExploder,
    room_sorter::{RoomSort, RoomSorter},
    simple::SimpleMapBuilder,
    voronoi::VoronoiCellBuilder,
    voronoi_spawning::VoronoiSpawning,
    waveform_collapse::WaveformCollapseBuilder,
};

mod area_starting_points;
mod automata;
mod bsp;
mod bsp_interior;
mod common;
mod cull_unreachable;
mod distant_exit;
mod dla;
mod drunkard;
mod maze;
mod prefab;
mod room_based_spawner;
mod room_based_stairs;
mod room_based_starting_position;
mod room_corner_rounding;
mod room_corridors_bsp;
mod room_corridors_dogleg;
mod room_draw;
mod room_exploder;
mod room_sorter;
mod simple;
mod themes;
mod voronoi;
mod voronoi_spawning;
mod waveform_collapse;

pub use themes::*;

pub struct BuilderMap {
    pub spawn_list: Vec<(Point, String)>,
    pub map: Map,
    pub starting_position: Option<Point>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl BuilderChain {
    fn new(depth: i32) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(depth),
                starting_position: None,
                rooms: None,
                history: Vec::new(),
            },
        }
    }

    fn initial(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder."),
        }
    }

    pub fn push(&mut self, metabuilder: Box<dyn MetaMapBuilder>) {
        self.builders.push(metabuilder)
    }

    pub fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting builder"),
            Some(starter) => {
                // Build the starting map.
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn.
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, ecs: &mut World) {
        for entity in self.build_data.spawn_list.iter() {
            spawn_entity(ecs, &(&entity.0, &entity.1));
        }
    }
}

fn random_initial_builder(rng: &mut RandomNumberGenerator) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 18);
    match builder {
        1 => (BSPDungeonBuilder::new(), true),
        2 => (BSPInteriorBuilder::new(), true),
        3 => (CellularAutomataBuilder::new(), false),
        4 => (DrunkardsWalkBuilder::open_area(), false),
        5 => (DrunkardsWalkBuilder::open_halls(), false),
        6 => (DrunkardsWalkBuilder::winding_passages(), false),
        7 => (DrunkardsWalkBuilder::fat_passages(), false),
        8 => (DrunkardsWalkBuilder::fearful_symmetry(), false),
        9 => (MazeBuilder::new(), false),
        10 => (DLABuilder::walk_inwards(), false),
        11 => (DLABuilder::walk_outwards(), false),
        12 => (DLABuilder::central_attractor(), false),
        13 => (DLABuilder::rorschach(), false),
        14 => (VoronoiCellBuilder::pythagoras(), false),
        15 => (VoronoiCellBuilder::manhattan(), false),
        16 => (VoronoiCellBuilder::chebyshev(), false),
        17 => (
            PrefabBuilder::constant(prefab::levels::WFC_POPULATED),
            false,
        ),
        _ => (SimpleMapBuilder::new(), true),
    }
}

fn random_room_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    let build_roll = rng.roll_dice(1, 3);
    match build_roll {
        1 => builder.initial(SimpleMapBuilder::new()),
        2 => builder.initial(BSPDungeonBuilder::new()),
        _ => builder.initial(BSPInteriorBuilder::new()),
    }

    // BSP interior still makes it own holes in the walls
    if build_roll != 3 {
        // Sort by one of the 5 available algorithms
        let sort_order = match rng.roll_dice(1, 5) {
            1 => RoomSort::Leftmost,
            2 => RoomSort::Rightmost,
            3 => RoomSort::Topmost,
            4 => RoomSort::Bottommost,
            _ => RoomSort::Central,
        };
        builder.push(RoomSorter::new(sort_order));
        builder.push(RoomDrawer::new());

        match rng.roll_dice(1, 2) {
            1 => builder.push(DoglegCorridors::new()),
            _ => builder.push(BSPCorridors::new()),
        }

        match rng.roll_dice(1, 6) {
            1 => builder.push(RoomExploder::new()),
            2 => builder.push(RoomCornerRounder::new()),
            _ => {}
        }
    }

    match rng.roll_dice(1, 2) {
        1 => builder.push(RoomBasedStartingPosition::new()),
        _ => {
            builder.push(AreaStartingPosition::new(
                XStart::random(rng),
                YStart::random(rng),
            ));
        }
    }

    match rng.roll_dice(1, 2) {
        1 => builder.push(RoomBasedStairs::new()),
        _ => builder.push(DistantExit::new()),
    }

    match rng.roll_dice(1, 2) {
        1 => builder.push(RoomBasedSpawner::new()),
        _ => builder.push(VoronoiSpawning::new()),
    }
}

pub fn random_shape_builder(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    match rng.roll_dice(1, 16) {
        1 => builder.initial(CellularAutomataBuilder::new()),
        2 => builder.initial(DrunkardsWalkBuilder::open_area()),
        3 => builder.initial(DrunkardsWalkBuilder::open_halls()),
        4 => builder.initial(DrunkardsWalkBuilder::winding_passages()),
        5 => builder.initial(DrunkardsWalkBuilder::fat_passages()),
        6 => builder.initial(DrunkardsWalkBuilder::fearful_symmetry()),
        7 => builder.initial(MazeBuilder::new()),
        8 => builder.initial(DLABuilder::walk_inwards()),
        9 => builder.initial(DLABuilder::walk_outwards()),
        10 => builder.initial(DLABuilder::central_attractor()),
        11 => builder.initial(DLABuilder::rorschach()),
        12 => builder.initial(VoronoiCellBuilder::pythagoras()),
        13 => builder.initial(VoronoiCellBuilder::manhattan()),
        _ => builder.initial(PrefabBuilder::constant(prefab::levels::WFC_POPULATED)),
    }

    cull_and_finalize(rng, builder);
}

fn cull_and_finalize(rng: &mut RandomNumberGenerator, builder: &mut BuilderChain) {
    // Set the start to the center and cull
    builder.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    builder.push(CullUnreachable::new());

    // Now pick a random starting area.
    builder.push(AreaStartingPosition::new(
        XStart::random(rng),
        YStart::random(rng),
    ));

    // Setup exit and spawn mobs
    builder.push(VoronoiSpawning::new());
    builder.push(DistantExit::new());
}

pub fn random_builder(new_depth: i32, rng: &mut RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    match rng.roll_dice(1, 2) {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder),
    }

    if rng.roll_dice(1, 3) == 1 {
        builder.push(WaveformCollapseBuilder::new());
        if rng.roll_dice(1, 5) == 1 {
            builder.push(CellularAutomataBuilder::new());
        }
        cull_and_finalize(rng, &mut builder);
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.push(PrefabBuilder::sectional(prefab::sections::UNDERGROUND_FORT));
    }

    builder.push(PrefabBuilder::vaults());

    builder

    // let mut builder = BuilderChain::new(new_depth);
    // builder.initial(SimpleMapBuilder::new());
    // builder.push(RoomDrawer::new());
    // builder.push(RoomSorter::new(RoomSort::Leftmost));
    // builder.push(BSPCorridors::new());
    // builder.push(RoomBasedSpawner::new());
    // builder.push(RoomBasedStairs::new());
    // builder.push(RoomBasedStartingPosition::new());
    // builder
}
