use crate::prelude::*;
use super::MapArchitect;

const MIN_ROOM_SIZE: i32 = 8;

#[derive(Default)]
pub struct BSPInteriorArchitect {
    rects: Vec<Rect>
}

impl BSPInteriorArchitect {
    fn build_rooms(&mut self, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        self.rects.clear();
        self.rects.push(Rect::with_size(1,1, MAP_WIDTH-2, MAP_HEIGHT-2));
        let first_room = self.rects[0];
        self.add_subrects(first_room, rng);

        let rooms = self.rects.clone();
        for r in rooms.iter() {
            let room = *r;
            mb.rooms.push(room);
            room.for_each(|pos| {
                if let Some(idx) = mb.map.try_idx(pos) {
                    mb.map.tiles[idx] = TileType::Floor;
                }
            });
            mb.take_snapshot();
        }

        for i in 0..mb.rooms.len()-1 {
            let room = mb.rooms[i];
            let next = mb.rooms[i+1];
            let start_x = rng.range(room.x1, room.x2-1);
            let start_y = rng.range(room.y1, room.y2-1);
            let end_x = rng.range(next.x1, next.x2-1);
            let end_y = rng.range(next.y1, next.y2-1);
            mb.build_corridor(Point::new(start_x, start_y), Point::new(end_x, end_y), rng);
            mb.take_snapshot();
        }
    }

    fn add_subrects(&mut self, rect: Rect, rng: &mut RandomNumberGenerator) {
        // Remove the last rect from the list
        self.rects.pop();

        // Calculate boundaries.
        let width = i32::abs(rect.x2 - rect.x1);
        let height = i32::abs(rect.y2 - rect.y1);
        let half_width = width/2;
        let half_height = height/2;

        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            // Horizontal split
            let h1 = Rect::with_size(rect.x1, rect.y1, half_width-1, height);
            self.rects.push(h1);
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h1, rng); }
            let h2 = Rect::with_size(rect.x1 + half_width+1, rect.y1, half_width-1, height);
            self.rects.push(h2);
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h2, rng); }
        }
        else {
            // Vertical split.
            let v1 = Rect::with_size(rect.x1, rect.y1, width, half_height-1);
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v1, rng); }
            let v2 = Rect::with_size(rect.x1, rect.y1 + half_height+1, width, half_height-1);
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v2, rng); }
        }
    }
}

impl MapArchitect for BSPInteriorArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.fill(TileType::Wall);
        mb.take_snapshot();

        self.build_rooms(&mut mb, rng);

        mb.generate_random_table();
        mb.player_start = mb.rooms[0].center();
        mb.goal_start = mb.rooms.last().unwrap().center();
        for room in mb.rooms.clone().iter().skip(1) {
            mb.spawn_room(room, rng);
        }

        mb
    }
}