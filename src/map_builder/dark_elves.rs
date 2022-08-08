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
