use crate::prelude::*;

#[derive(Default)]
pub struct BSPCorridors {}

impl MetaMapBuilder for BSPCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build_corridors(rng, build_data);
    }
}

impl BSPCorridors {
    #[allow(dead_code)]
    pub fn new() -> Box<BSPCorridors> {
        Box::new(BSPCorridors::default())
    }

    fn build_corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        use super::common::draw_corridor;

        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("BSP Corridors require a builder with room structures");
        }

        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next_room = rooms[i + 1];
            let start = Point::new(
                room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1),
                room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1),
            );
            let end = Point::new(
                next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1),
                next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1),
            );

            draw_corridor(&mut build_data.map, start, end);
            build_data.take_snapshot();
        }
    }
}
