use crate::prelude::*;
use super::MapArchitect;

const MAX_SPAWNS: i32 = 4;

pub struct RoomsArchitect {}

impl MapArchitect for RoomsArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;

        mb.fill(TileType::Wall);
        mb.build_random_rooms(rng);
        mb.build_corridors(rng);
        mb.generate_random_table();
        mb.player_start = mb.rooms[0].center();
        mb.goal_start = mb.rooms.last().unwrap().center();
        for room in mb.rooms.clone().iter().skip(1) {
            spawn_room(&room, &mut mb, rng);
        }

        mb
    }
}

fn spawn_room(room: &Rect, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
    let num_spawns = rng.roll_dice(1, MAX_SPAWNS + 3) + mb.depth - 3;
    let mut spawnable_tiles = Vec::from_iter(room.point_set());

    for _ in 0 .. num_spawns {
        if spawnable_tiles.is_empty() {
            break;
        }
        let target_index = rng.random_slice_index(&spawnable_tiles)
            .unwrap();
        mb.spawns.push(spawnable_tiles[target_index].clone());
        spawnable_tiles.remove(target_index);
    }
}