use crate::prelude::*;

use super::common::build_corridor;

const MIN_ROOM_SIZE: i32 = 8;

#[derive(Default)]
pub struct BSPInteriorBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BSPInteriorBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Wall);
        self.build_rooms(rng, build_data);
    }
}

impl BSPInteriorBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<BSPInteriorBuilder> {
        Box::new(BSPInteriorBuilder::default())
    }

    fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.rects.clear();
        self.rects.push(Rect::with_size(
            1,
            1,
            build_data.map.width - 2,
            build_data.map.height - 2,
        ));
        let first_room = self.rects[0];
        self.add_subrects(first_room, rng);

        let rooms = self.rects.clone();
        for r in rooms.iter() {
            r.for_each(|pos| {
                if let Some(idx) = build_data.map.try_idx(pos) {
                    build_data.map.tiles[idx] = TileType::Floor;
                }
            });
            build_data.take_snapshot();
        }

        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next = rooms[i + 1];
            let start = Point::new(
                rng.range(room.x1, room.x2 - 1),
                rng.range(room.y1, room.y2 - 1),
            );
            let end = Point::new(
                rng.range(next.x1, next.x2 - 1),
                rng.range(next.y1, next.y2 - 1),
            );

            build_corridor(&mut build_data.map, rng, start, end);
            build_data.take_snapshot();
        }

        build_data.rooms = Some(rooms);
    }

    fn add_subrects(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) {
        // Remove the last rect from the list
        self.rects.pop();

        // Calculate boundaries.
        let width = i32::abs(rect.x2 - rect.x1);
        let height = i32::abs(rect.y2 - rect.y1);
        let half_width = width / 2;
        let half_height = height / 2;

        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            // Horizontal split
            let h1 = Rect::with_size(rect.x1, rect.y1, half_width - 1, height);
            self.rects.push(h1);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h1, rng);
            }
            let h2 = Rect::with_size(rect.x1 + half_width + 1, rect.y1, half_width - 1, height);
            self.rects.push(h2);
            if half_width > MIN_ROOM_SIZE {
                self.add_subrects(h2, rng);
            }
        } else {
            // Vertical split.
            let v1 = Rect::with_size(rect.x1, rect.y1, width, half_height - 1);
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v1, rng);
            }
            let v2 = Rect::with_size(rect.x1, rect.y1 + half_height + 1, width, half_height - 1);
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE {
                self.add_subrects(v2, rng);
            }
        }
    }
}
