use crate::prelude::*;

#[derive(Default)]
pub struct DoglegCorridors {}

impl MetaMapBuilder for DoglegCorridors {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl DoglegCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<DoglegCorridors> {
        Box::new(DoglegCorridors::default())
    }

    fn corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        use super::common::*;

        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Dogleg corridors require a builder with room structures");
        }

        for (i, room) in rooms.iter().enumerate().skip(1) {
            let start = rooms[i - 1].center();
            let end = room.center();
            build_corridor(&mut build_data.map, rng, start, end);
            build_data.take_snapshot();
        }
    }
}
