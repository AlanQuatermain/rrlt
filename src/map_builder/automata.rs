use crate::prelude::*;

#[derive(Default)]
pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Wall);
        self.build(rng, build_data);
    }
}

impl MetaMapBuilder for CellularAutomataBuilder {
    #[allow(dead_code)]
    fn build_map(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.iteration(&mut build_data.map);
        build_data.take_snapshot();
    }
}

impl CellularAutomataBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder::default())
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.random_noise_map(rng, &mut build_data.map);
        build_data.take_snapshot();

        for _ in 0..15 {
            self.iteration(&mut build_data.map);
            build_data.take_snapshot();
        }
    }

    fn random_noise_map(&self, rng: &mut RandomNumberGenerator, map: &mut Map) {
        for y in 1..map.height as i32 - 1 {
            for x in 1..map.width as i32 - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = map.point2d_to_index(Point::new(x, y));
                if roll > 55 {
                    map.tiles[idx] = TileType::Floor;
                } else {
                    map.tiles[idx] = TileType::Wall;
                }
            }
        }
    }

    fn count_neighbors(&self, x: i32, y: i32, map: &Map) -> usize {
        map.count_neighbors(Point::new(x, y))
    }

    fn iteration(&self, map: &mut Map) {
        let mut new_tiles = map.tiles.clone();
        for y in 1..map.height as i32 - 1 {
            for x in 1..map.width as i32 - 1 {
                let neighbors = self.count_neighbors(x, y, map);
                let idx = map.point2d_to_index(Point::new(x, y));
                if neighbors > 4 || neighbors == 0 {
                    new_tiles[idx] = TileType::Wall;
                } else {
                    new_tiles[idx] = TileType::Floor;
                }
            }
        }
        map.tiles = new_tiles;
    }

    fn __find_start(&self, map: &Map) -> Point {
        map.find_closest_floor(Point::new(map.width / 2, map.height / 2))
    }
}
