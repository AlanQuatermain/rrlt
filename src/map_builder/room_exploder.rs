use crate::prelude::*;

use super::common::{paint, Symmetry};

#[derive(Default)]
pub struct RoomExploder {}

impl MetaMapBuilder for RoomExploder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomExploder {
    #[allow(dead_code)]
    pub fn new() -> Box<RoomExploder> {
        Box::new(RoomExploder::default())
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room explosions require a builder with room structures");
        }

        for room in rooms.iter() {
            let start = room.center();
            let n_diggers = rng.roll_dice(1, 20) - 5;
            if n_diggers <= 0 {
                continue;
            }

            for _ in 0..n_diggers {
                let mut drunk_pos = start;
                let mut life = 20;
                let mut did_something = false;

                while life > 0 {
                    let idx = build_data.map.point2d_to_index(drunk_pos);
                    if build_data.map.tiles[idx] == TileType::Wall {
                        did_something = true;
                    }
                    paint(&mut build_data.map, Symmetry::None, 1, drunk_pos);
                    build_data.map.tiles[idx] = TileType::DownStairs;

                    match rng.roll_dice(1, 4) {
                        1 => {
                            if drunk_pos.x > 2 {
                                drunk_pos.x -= 1
                            }
                        }
                        2 => {
                            if drunk_pos.x < build_data.map.width as i32 - 2 {
                                drunk_pos.x += 1
                            }
                        }
                        3 => {
                            if drunk_pos.y > 2 {
                                drunk_pos.y -= 1
                            }
                        }
                        _ => {
                            if drunk_pos.y < build_data.map.height as i32 - 2 {
                                drunk_pos.y += 1
                            }
                        }
                    }

                    life -= 1;
                }
                if did_something {
                    build_data.take_snapshot();
                }

                for t in build_data.map.tiles.iter_mut() {
                    if *t == TileType::DownStairs {
                        *t = TileType::Floor;
                    }
                }
            }
        }
    }
}
