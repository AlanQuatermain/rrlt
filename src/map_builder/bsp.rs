use crate::prelude::*;

#[derive(Default)]
pub struct BSPDungeonBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BSPDungeonBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Wall);
        self.build_rooms(rng, build_data);
    }
}

impl BSPDungeonBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<BSPDungeonBuilder> {
        Box::new(BSPDungeonBuilder::default())
    }

    fn build_rooms(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut rooms: Vec<Rect> = Vec::new();
        self.rects.clear();
        self.rects.push(Rect::with_size(
            2,
            2,
            build_data.map.width - 4,
            build_data.map.height - 4,
        ));
        let first_room = self.rects[0];
        self.add_subrects(first_room); // Divide the first room.

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        for _ in 0..240 {
            let rect = self.get_random_rect(rng);
            let candidate = self.get_random_sub_rect(rect, rng);

            if self.is_possible(candidate, &build_data.map, &rooms) {
                rooms.push(candidate);
                self.add_subrects(rect);
            }
        }
        build_data.rooms = Some(rooms);
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

    fn is_possible(&self, rect: Rect, map: &Map, rooms: &Vec<Rect>) -> bool {
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.y1 -= 2;
        expanded.x2 += 2;
        expanded.y2 += 2;

        let mut can_build = true;

        for r in rooms.iter() {
            if r.intersect(&rect) {
                return false;
            }
        }

        for y in expanded.y1..expanded.y2 {
            for x in expanded.x1..expanded.x2 {
                if x < 1 || y < 1 || x as usize > map.width - 2 || y as usize > map.height - 2 {
                    return false;
                }
            }
        }

        true
    }
}
