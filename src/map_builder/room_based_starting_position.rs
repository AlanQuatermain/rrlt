use crate::prelude::*;

#[derive(Default)]
pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl RoomBasedStartingPosition {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomBasedStartingPosition> {
        Box::new(RoomBasedStartingPosition::default())
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let start_pos = rooms[0].center();
            build_data.starting_position = Some(start_pos);
        } else {
            panic!("Room Based Starting Position only works after rooms have been created");
        }
    }
}
