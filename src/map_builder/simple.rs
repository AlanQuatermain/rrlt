use super::{BuilderMap, InitialMapBuilder};
use crate::{
    map_builder::common::{
        apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel, build_corridor,
    },
    prelude::*,
};

#[derive(Default)]
pub struct SimpleMapBuilder {}

impl InitialMapBuilder for SimpleMapBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Wall);
        self.rooms_and_corridors(rng, build_data);
    }
}

impl SimpleMapBuilder {
    pub fn new() -> Box<SimpleMapBuilder> {
        Box::new(SimpleMapBuilder::default())
    }

    fn rooms_and_corridors(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        let mut rooms: Vec<Rect> = Vec::new();

        for i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, build_data.map.width as i32 - w - 1) - 1;
            let y = rng.roll_dice(1, build_data.map.height as i32 - h - 1) - 1;

            let new_room = Rect::with_size(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                    break;
                }
            }
            if ok {
                apply_room_to_map(&mut build_data.map, &new_room);
                build_data.take_snapshot();

                if !rooms.is_empty() {
                    let new = new_room.center();
                    let prev = rooms.last().unwrap().center();
                    build_corridor(&mut build_data.map, rng, prev, new);
                }

                rooms.push(new_room);
                build_data.take_snapshot();
            }
        }

        build_data.rooms = Some(rooms);
    }
}
