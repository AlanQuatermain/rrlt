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
    room_exploder::RoomExploder,
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
mod room_exploder;
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

    fn initial(&mut self, starter: Box<dyn InitialMapBuilder>) -> &Self {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder."),
        };
        self
    }

    pub fn start_with(starter: Box<dyn InitialMapBuilder>, depth: i32) -> Self {
        BuilderChain {
            starter: Some(starter),
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

pub fn random_builder(new_depth: i32, rng: &mut RandomNumberGenerator) -> BuilderChain {
    // let (random_starter, has_rooms) = random_initial_builder(rng);
    // let mut builder = BuilderChain::start_with(random_starter, new_depth);
    // if has_rooms {
    //     builder.push(RoomBasedSpawner::new());
    //     builder.push(RoomBasedStairs::new());
    //     builder.push(RoomBasedStartingPosition::new());
    // } else {
    //     builder.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    //     builder.push(CullUnreachable::new());
    //     builder.push(VoronoiSpawning::new());
    //     builder.push(DistantExit::new());
    // }

    // if rng.roll_dice(1, 3) == 1 {
    //     builder.push(WaveformCollapseBuilder::new());
    // }

    // if rng.roll_dice(1, 20) == 1 {
    //     builder.push(PrefabBuilder::sectional(prefab::sections::UNDERGROUND_FORT));
    // }

    // builder.push(PrefabBuilder::vaults());

    // builder

    let mut builder = BuilderChain::start_with(BSPDungeonBuilder::new(), new_depth);
    builder.push(RoomCornerRounder::new());
    builder.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    builder.push(CullUnreachable::new());
    builder.push(VoronoiSpawning::new());
    builder.push(DistantExit::new());
    builder
}
