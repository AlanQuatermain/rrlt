use crate::prelude::*;

use super::{
    area_ending_points::{AreaEndingPosition, XEnd, YEnd},
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    automata::CellularAutomataBuilder,
    cull_unreachable::CullUnreachable,
    prefab::{
        sections::{DROW_ENTRY, UNDERGROUND_FORT},
        PrefabBuilder,
    },
    voronoi_spawning::VoronoiSpawning,
    waveform_collapse::WaveformCollapseBuilder,
};

pub fn mushroom_entrance(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Mushroom Grove");
    chain.build_data.map.theme = MapTheme::transition(
        MapTheme::MushroomGrove,
        MapTheme::Dungeon,
        0.8,
        Orientation::Horizontal,
    );
    chain.build_data.map.outdoors = false;

    chain.initial(CellularAutomataBuilder::new());
    chain.push(WaveformCollapseBuilder::new());
    chain.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.push(CullUnreachable::new());
    chain.push(AreaStartingPosition::new(XStart::Right, YStart::Center));
    chain.push(AreaEndingPosition::new(XEnd::Left, YEnd::Center));
    chain.push(VoronoiSpawning::new());
    chain.push(PrefabBuilder::sectional(UNDERGROUND_FORT));

    chain
}

pub fn mushroom_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Mushroom Grove");
    chain.build_data.map.theme = MapTheme::MushroomGrove;
    chain.build_data.map.outdoors = false;

    chain.initial(CellularAutomataBuilder::new());
    chain.push(WaveformCollapseBuilder::new());
    chain.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.push(CullUnreachable::new());
    chain.push(AreaStartingPosition::new(XStart::Right, YStart::Center));
    chain.push(AreaEndingPosition::new(XEnd::Left, YEnd::Center));
    chain.push(VoronoiSpawning::new());

    chain
}

pub fn mushroom_exit(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: usize,
    height: usize,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dark Elf Borders");
    chain.build_data.map.theme = MapTheme::MushroomGrove;
    chain.build_data.map.outdoors = false;

    chain.initial(CellularAutomataBuilder::new());
    chain.push(WaveformCollapseBuilder::new());
    chain.push(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.push(CullUnreachable::new());
    chain.push(AreaStartingPosition::new(XStart::Right, YStart::Center));
    chain.push(AreaEndingPosition::new(XEnd::Left, YEnd::Center));
    chain.push(VoronoiSpawning::new());
    chain.push(PrefabBuilder::sectional(DROW_ENTRY));

    chain
}
