use crate::prelude::*;

#[allow(dead_code)]
pub enum XEnd {
    Left,
    Center,
    Right,
}

#[allow(dead_code)]
pub enum YEnd {
    Top,
    Center,
    Bottom,
}

pub struct AreaEndingPosition {
    x: XEnd,
    y: YEnd,
}

impl MetaMapBuilder for AreaEndingPosition {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl AreaEndingPosition {
    #[allow(dead_code)]
    pub fn new(x: XEnd, y: YEnd) -> Box<AreaEndingPosition> {
        Box::new(AreaEndingPosition { x, y })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut seed = Point::zero();

        match self.x {
            XEnd::Left => seed.x = 1,
            XEnd::Center => seed.x = build_data.map.width as i32 / 2,
            XEnd::Right => seed.x = build_data.map.width as i32 - 2,
        }
        match self.y {
            YEnd::Top => seed.y = 1,
            YEnd::Center => seed.y = build_data.map.height as i32 / 2,
            YEnd::Bottom => seed.y = build_data.map.height as i32 - 2,
        }

        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if tiletype.is_walkable() {
                available_floors.push((
                    idx,
                    DistanceAlg::PythagorasSquared
                        .distance2d(build_data.map.index_to_point2d(idx), seed),
                ));
            }
        }

        if available_floors.is_empty() {
            panic!("No valid floors to place exit");
        }

        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        build_data.map.tiles[available_floors[0].0] = TileType::DownStairs;
    }
}
