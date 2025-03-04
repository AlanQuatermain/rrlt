use crate::prelude::*;

use self::{
    area_starting_points::*,
    automata::CellularAutomataBuilder,
    bsp::BSPDungeonBuilder,
    bsp_interior::BSPInteriorBuilder,
    caverns::*,
    cull_unreachable::CullUnreachable,
    dark_elves::{dark_elf_city, dark_elf_plaza},
    distant_exit::DistantExit,
    dla::DLABuilder,
    door_placement::DoorPlacement,
    drunkard::DrunkardsWalkBuilder,
    dwarf_fort::dwarf_fort_builder,
    maze::MazeBuilder,
    mushroom::{mushroom_builder, mushroom_entrance, mushroom_exit},
    nearest_corridors::NearestCorridors,
    prefab::PrefabBuilder,
    room_based_spawner::RoomBasedSpawner,
    room_based_stairs::RoomBasedStairs,
    room_based_starting_position::RoomBasedStartingPosition,
    room_corner_rounding::RoomCornerRounder,
    room_corridor_spawner::CorridorSpawner,
    room_corridors_bsp::BSPCorridors,
    room_corridors_dogleg::DoglegCorridors,
    room_corridors_lines::StraightLineCorridors,
    room_draw::RoomDrawer,
    room_exploder::RoomExploder,
    room_sorter::{RoomSort, RoomSorter},
    simple::SimpleMapBuilder,
    voronoi::VoronoiCellBuilder,
    voronoi_spawning::VoronoiSpawning,
    waveform_collapse::WaveformCollapseBuilder,
};

mod area_ending_points;
mod area_starting_points;
mod automata;
mod bsp;
mod bsp_interior;
mod cave_decorator;
mod caverns;
mod common;
mod cull_unreachable;
mod dark_elves;
mod distant_exit;
mod dla;
mod door_placement;
mod drunkard;
mod dwarf_fort;
mod forest;
mod maze;
mod mushroom;
mod nearest_corridors;
mod prefab;
mod room_based_spawner;
mod room_based_stairs;
mod room_based_starting_position;
mod room_corner_rounding;
mod room_corridor_spawner;
mod room_corridors_bsp;
mod room_corridors_dogleg;
mod room_corridors_lines;
mod room_draw;
mod room_exploder;
mod room_sorter;
mod simple;
mod themes;
mod town;
mod voronoi;
mod voronoi_spawning;
mod waveform_collapse;

use forest::forest_builder;
pub use themes::*;
use town::town_builder;

const PRINT_CHAIN_ITEMS: bool = false;

pub struct BuilderMap {
    pub spawn_list: Vec<(Point, String)>,
    pub map: Map,
    pub starting_position: Option<Point>,
    pub rooms: Option<Vec<Rect>>,
    pub corridors: Option<Vec<Vec<usize>>>,
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
            for v in snapshot.visible_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl BuilderChain {
    fn new<S: ToString>(depth: i32, width: usize, height: usize, name: S) -> BuilderChain {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(depth, width, height, name),
                starting_position: None,
                rooms: None,
                corridors: None,
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

    pub fn spawn_entities(&mut self, ecs: &mut World, dm: &MasterDungeonMap) {
        for entity in self.build_data.spawn_list.iter() {
            spawn_entity(ecs, dm, &(&entity.0, &entity.1));
        }
    }

    pub fn debug_print(&self) {
        println!("Build chain:");
        println!("  {}", type_of(&self.starter));
        for builder in &self.builders {
            println!("  {}", type_of(builder));
        }
    }
}

fn type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
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

        match rng.roll_dice(1, 4) {
            1 => builder.push(DoglegCorridors::new()),
            2 => builder.push(BSPCorridors::new()),
            3 => builder.push(StraightLineCorridors::new()),
            _ => builder.push(NearestCorridors::new()),
        }

        if rng.roll_dice(1, 2) == 1 {
            builder.push(CorridorSpawner::new());
        }

        match rng.roll_dice(1, 8) {
            1 => builder.push(RoomExploder::new()),
            2 => builder.push(RoomCornerRounder::new()),
            3 => builder.push(DLABuilder::heavy_erosion()),
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
    match rng.roll_dice(1, 15) {
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
        14 => builder.initial(VoronoiCellBuilder::chebyshev()),
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

pub fn random_builder(
    new_depth: i32,
    width: usize,
    height: usize,
    rng: &mut RandomNumberGenerator,
) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height, "New Map");
    match rng.roll_dice(1, 2) {
        1 => random_room_builder(rng, &mut builder),
        _ => random_shape_builder(rng, &mut builder),
    }

    if rng.roll_dice(1, 6) == 1 {
        builder.push(WaveformCollapseBuilder::new());
        // if rng.roll_dice(1, 5) == 1 {
        //     builder.push(CellularAutomataBuilder::new());
        // }
        cull_and_finalize(rng, &mut builder);
    }

    if rng.roll_dice(1, 20) == 1 {
        builder.push(PrefabBuilder::sectional(prefab::sections::UNDERGROUND_FORT));
    }

    builder.push(DoorPlacement::new());
    builder.push(PrefabBuilder::vaults());

    if PRINT_CHAIN_ITEMS {
        builder.debug_print();
    }

    builder

    // let mut builder = BuilderChain::new(new_depth, width, height);
    // builder.initial(SimpleMapBuilder::new());
    // builder.push(RoomDrawer::new());
    // builder.push(RoomSorter::new(RoomSort::Leftmost));
    // builder.push(BSPCorridors::new());
    // builder.push(RoomBasedSpawner::new());
    // builder.push(CorridorSpawner::new());
    // builder.push(RoomBasedStairs::new());
    // builder.push(RoomBasedStartingPosition::new());
    // builder

    // builder.initial(BSPInteriorBuilder::new());
    // builder.push(DoorPlacement::new());
    // builder.push(RoomBasedSpawner::new());
    // builder.push(RoomBasedStairs::new());
    // builder.push(RoomBasedStartingPosition::new());
    // builder
}

pub fn level_builder(
    new_depth: i32,
    width: usize,
    height: usize,
    rng: &mut RandomNumberGenerator,
) -> BuilderChain {
    match new_depth {
        0 => town_builder(new_depth, width, height, rng),
        1 => forest_builder(new_depth, width, height, rng),
        2 => limestone_cavern_builder(new_depth, rng, width, height),
        3 => limestone_deep_cavern_builder(new_depth, rng, width, height),
        4 => limestone_transition_builder(new_depth, rng, width, height),
        5 => dwarf_fort_builder(new_depth, rng, width, height),
        6 => mushroom_entrance(new_depth, rng, width, height),
        7 => mushroom_builder(new_depth, rng, width, height),
        8 => mushroom_exit(new_depth, rng, width, height),
        9 => dark_elf_city(new_depth, rng, width, height),
        10 => dark_elf_plaza(new_depth, rng, width, height),
        _ => random_builder(new_depth, width, height, rng),
    }
}
