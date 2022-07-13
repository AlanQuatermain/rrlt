use crate::prelude::*;
use super::MapArchitect;

pub struct EmptyArchitect {}

impl MapArchitect for EmptyArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.take_snapshot();
        mb.fill(TileType::Floor);
        mb.take_snapshot();
        mb.player_start = Point::new(MAP_WIDTH/2, MAP_HEIGHT/2);
        mb.goal_start = mb.find_most_distant();
        for _ in 0..50 {
            mb.spawns.push(
                Point::new(
                    rng.range(1, MAP_WIDTH),
                    rng.range(1, MAP_HEIGHT)
                )
            )
        }
        mb
    }
}