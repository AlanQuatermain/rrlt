use crate::prelude::*;

#[derive(Default, Debug)]
pub struct CaveDecorator {}

impl MetaMapBuilder for CaveDecorator {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveDecorator {
    #[allow(dead_code)]
    pub fn new() -> Box<CaveDecorator> {
        Box::new(CaveDecorator::default())
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let old_map = build_data.map.clone();
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            // Gravel spawning
            if *tt == TileType::Floor && rng.roll_dice(1, 6) == 1 {
                *tt = TileType::Gravel;
            } else if *tt == TileType::Floor && rng.roll_dice(1, 10) == 1 {
                // Spawn passable pools,
                *tt = TileType::ShallowWater;
            } else if *tt == TileType::Wall {
                // Spawn deep pools and stalactites.
                let pos = old_map.index_to_point2d(idx);
                match old_map.count_neighbors(pos) {
                    2 => *tt = TileType::DeepWater,
                    1 => match rng.roll_dice(1, 4) {
                        1 => *tt = TileType::Stalactite,
                        2 => *tt = TileType::Stalagmite,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
}
