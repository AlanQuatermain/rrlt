use crate::prelude::*;

#[derive(Default)]
pub struct CullUnreachable {}

impl MetaMapBuilder for CullUnreachable {
    fn build_map(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl CullUnreachable {
    #[allow(dead_code)]
    pub fn new() -> Box<CullUnreachable> {
        Box::new(CullUnreachable::default())
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.point2d_to_index(starting_pos);
        build_data.map.populate_blocked();

        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = DijkstraMap::new(
            build_data.map.width,
            build_data.map.height,
            &map_starts,
            &build_data.map,
            1000.0,
        );
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance = dijkstra_map.map[i];
                // We can't get to this tile, so make it a wall.
                if distance == std::f32::MAX {
                    *tile = TileType::Wall;
                }
            }
        }
    }
}
