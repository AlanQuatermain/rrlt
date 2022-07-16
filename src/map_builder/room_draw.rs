use crate::prelude::*;

#[derive(Default)]
pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomDrawer {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomDrawer> {
        Box::new(RoomDrawer::default())
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room Bounding requires a builder with room structures");
        }

        for room in rooms.iter() {
            match rng.roll_dice(1, 4) {
                1 => self.circle(build_data, room),
                _ => self.rectangle(build_data, room),
            }
            build_data.take_snapshot();
        }
    }

    fn rectangle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        room.for_each(|pos| {
            let idx = build_data.map.point2d_to_index(pos);
            build_data.map.tiles[idx] = TileType::Floor;
        });
    }

    fn circle(&mut self, build_data: &mut BuilderMap, room: &Rect) {
        let radius = i32::min(room.width(), room.height()) as f32 / 2.0;
        let center = room.center();

        room.for_each(|pos| {
            let idx = build_data.map.point2d_to_index(pos);
            let distance = DistanceAlg::Pythagoras.distance2d(center, pos);
            if distance <= radius {
                build_data.map.tiles[idx] = TileType::Floor;
            }
        });
    }
}
