use crate::prelude::*;

#[derive(Default)]
pub struct RoomCornerRounder {}

impl MetaMapBuilder for RoomCornerRounder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(build_data);
    }
}

impl RoomCornerRounder {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomCornerRounder> {
        Box::new(RoomCornerRounder::default())
    }

    fn fill_if_corner(&mut self, pos: Point, build_data: &mut BuilderMap) {
        let w = build_data.map.width;
        let h = build_data.map.height;
        let idx = build_data.map.point2d_to_index(pos);
        let mut neighbor_walls = 0;
        if pos.x > 0 && build_data.map.tiles[idx - 1] == TileType::Wall {
            neighbor_walls += 1
        }
        if pos.y > 0 && build_data.map.tiles[idx - w] == TileType::Wall {
            neighbor_walls += 1
        }
        if pos.x < w as i32 - 2 && build_data.map.tiles[idx + 1] == TileType::Wall {
            neighbor_walls += 1
        }
        if pos.y < h as i32 - 2 && build_data.map.tiles[idx + w] == TileType::Wall {
            neighbor_walls += 1
        }

        if neighbor_walls == 2 {
            build_data.map.tiles[idx] = TileType::Wall;
        }
    }

    fn build(&mut self, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room Rounding requires a builder with room structures.");
        }

        for room in rooms.iter() {
            self.fill_if_corner(Point::new(room.x1, room.y1), build_data);
            self.fill_if_corner(Point::new(room.x2, room.y1), build_data);
            self.fill_if_corner(Point::new(room.x1, room.y2), build_data);
            self.fill_if_corner(Point::new(room.x2, room.y2), build_data);

            build_data.take_snapshot();
        }
    }
}
