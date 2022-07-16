use std::collections::HashSet;

use super::{common::*, MetaMapBuilder};
use crate::prelude::*;

#[derive(Default)]
pub struct StraightLineCorridors {}

impl MetaMapBuilder for StraightLineCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl StraightLineCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<StraightLineCorridors> {
        Box::new(StraightLineCorridors::default())
    }

    fn corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Straight line corridors require a builder with room structures");
        }

        let mut corridors = Vec::new();
        let mut connected: HashSet<usize> = HashSet::new();
        for (i, room) in rooms.iter().enumerate() {
            let mut room_distance: Vec<(usize, f32)> = Vec::new();
            let room_center = room.center();
            for (j, other_room) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_center = other_room.center();
                    let distance = DistanceAlg::Pythagoras.distance2d(room_center, other_center);
                    room_distance.push((j, distance));
                }
            }

            if !room_distance.is_empty() {
                room_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let dest_center = rooms[room_distance[0].0].center();

                let mut corridor = Vec::new();
                let line = line2d(LineAlg::Bresenham, room_center, dest_center);
                for cell in line.iter() {
                    let idx = build_data.map.point2d_to_index(*cell);
                    build_data.map.tiles[idx] = TileType::Floor;
                    corridor.push(idx);
                }
                corridors.push(corridor);

                connected.insert(i);
                build_data.take_snapshot();
            }
        }

        build_data.corridors = Some(corridors);
    }
}
