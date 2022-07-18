use super::MapArchitect;
use crate::prelude::*;

pub struct EmptyArchitect {}

impl MapArchitect for EmptyArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder {
        let mut mb = MapBuilder::default();
        mb.depth = depth;
        mb.take_snapshot();
        mb.fill(TileType::Floor);
        mb.take_snapshot();
        mb.player_start = Point::new(map.width / 2, map.height / 2);
        mb.goal_start = mb.find_most_distant();
        for _ in 0..50 {
            mb.spawns.push(Point::new(
                rng.range(1, map.width),
                rng.range(1, map.height),
            ))
        }
        mb
    }

    fn spawn(&mut self, ecs: &mut World, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
        for pos in mb.spawns.iter() {
            spawn_mob(ecs, *pos, &mb.random_table, rng)
        }
    }
}
