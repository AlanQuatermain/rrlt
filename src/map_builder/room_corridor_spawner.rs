use super::{common::*, MetaMapBuilder};
use crate::prelude::*;

#[derive(Default)]
pub struct CorridorSpawner {}

impl MetaMapBuilder for CorridorSpawner {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CorridorSpawner {
    #[allow(dead_code)]
    pub fn new() -> Box<CorridorSpawner> {
        Box::new(CorridorSpawner::default())
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        if let Some(corridors) = &build_data.corridors {
            for c in corridors.iter() {
                let points: Vec<Point> = c
                    .iter()
                    .map(|idx| build_data.map.index_to_point2d(*idx))
                    .collect();
                fill_spawns_for_region(&build_data.map, rng, &points, &mut build_data.spawn_list);
            }
        }
    }
}
