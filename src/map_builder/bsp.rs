use super::MapArchitect;
use crate::prelude::*;

#[derive(Default)]
pub struct BSPArchitect {
    rects: Vec<Rect>,
}

impl BSPArchitect {
    fn build_rooms(&mut self, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        self.rects.clear();
        self.rects
            .push(Rect::with_size(2, 2, MAP_WIDTH - 4, MAP_HEIGHT - 4));
        let first_room = self.rects[0];
        self.add_subrects(first_room); // Divide the first room.

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        for _ in 0..240 {
            let rect = self.get_random_rect(rng);
            let candidate = self.get_random_sub_rect(rect, rng);

            if self.is_possible(candidate, &mb.map) {
                candidate.for_each(|pos| {
                    let idx = mb.map.point2d_to_index(pos);
                    mb.map.tiles[idx] = TileType::Floor;
                });
                mb.rooms.push(candidate);
                self.add_subrects(rect);
                mb.take_snapshot();
            }
        }
    }

    fn add_subrects(&mut self, rect: Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects
            .push(Rect::with_size(rect.x1, rect.y1, half_width, half_height));
        self.rects.push(Rect::with_size(
            rect.x1,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::with_size(
            rect.x1 + half_width,
            rect.y1,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::with_size(
            rect.x1 + half_width,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
    }

    fn get_random_rect(&self, rng: &mut RandomNumberGenerator) -> Rect {
        let idx = rng.random_slice_index(self.rects.as_slice()).unwrap();
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect: Rect, rng: &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10)) - 1) + 1;
        let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10)) - 1) + 1;

        result.x1 += rng.roll_dice(1, 6) - 1;
        result.y1 += rng.roll_dice(1, 6) - 1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect: Rect, map: &Map) -> bool {
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.y1 -= 2;
        expanded.x2 += 2;
        expanded.y2 += 2;

        let mut can_build = true;
        for y in expanded.y1..expanded.y2 {
            for x in expanded.x1..expanded.x2 {
                if x < 1 || y < 1 || x as usize > MAP_WIDTH - 2 || y as usize > MAP_HEIGHT - 2 {
                    can_build = false;
                    break;
                }
                let idx = map.point2d_to_index(Point::new(x, y));
                if map.tiles[idx] != TileType::Wall {
                    can_build = false;
                    break;
                }
            }
            if !can_build {
                break;
            }
        }

        can_build
    }

    fn build_corridors(&self, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        let mut rooms = mb.rooms.clone();
        rooms.sort_by(|a, b| a.x1.cmp(&b.x1));

        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next_room = rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x =
                next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1);
            let end_y =
                next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1);

            mb.build_corridor(Point::new(start_x, start_y), Point::new(end_x, end_y), rng);
            mb.take_snapshot();
        }
    }
}

impl MapArchitect for BSPArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.fill(TileType::Wall);
        mb.take_snapshot();

        self.build_rooms(&mut mb, rng);
        self.build_corridors(&mut mb, rng);

        mb.generate_random_table();
        mb.player_start = mb.rooms[0].center();
        mb.goal_start = mb.rooms.last().unwrap().center();

        mb
    }

    fn spawn(&mut self, _ecs: &mut World, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        for room in mb.rooms.clone().iter().skip(1) {
            mb.spawn_room(room, rng);
        }
    }
}
