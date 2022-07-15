use super::MapArchitect;
use crate::prelude::*;

#[derive(Default)]
pub struct RoomsArchitect {}

impl MapArchitect for RoomsArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;

        mb.fill(TileType::Wall);
        mb.take_snapshot();
        mb.build_random_rooms(rng);
        mb.build_corridors(rng);
        mb.generate_random_table();
        mb.player_start = mb.rooms[0].center();
        mb.goal_start = mb.rooms.last().unwrap().center();

        mb
    }

    fn spawn(&mut self, _ecs: &mut World, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        for room in mb.rooms.clone().iter().skip(1) {
            mb.spawn_room(&room, rng);
        }
    }
}
