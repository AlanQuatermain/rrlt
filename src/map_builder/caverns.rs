use crate::prelude::*;

use super::{
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    cave_decorator::CaveDecorator,
    cull_unreachable::CullUnreachable,
    distant_exit::DistantExit,
    dla::DLABuilder,
    drunkard::DrunkardsWalkBuilder,
    prefab::{self, PrefabBuilder},
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
