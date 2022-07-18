use crate::prelude::*;

#[derive(PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

pub struct VoronoiCellBuilder {
    algorithm: DistanceAlgorithm,
    num_seeds: usize,
}

impl InitialMapBuilder for VoronoiCellBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.fill(TileType::Wall);
        self.build(rng, build_data);
    }
}

impl VoronoiCellBuilder {
    pub fn pythagoras() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder {
            algorithm: DistanceAlgorithm::Pythagoras,
            num_seeds: 64,
        })
    }

    pub fn manhattan() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder {
            algorithm: DistanceAlgorithm::Manhattan,
            num_seeds: 64,
        })
    }

    pub fn chebyshev() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder {
            algorithm: DistanceAlgorithm::Chebyshev,
            num_seeds: 64,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let seeds = self.generate_seeds(
            build_data.map.width as i32,
            build_data.map.height as i32,
            rng,
            &build_data.map,
        );
        let membership = self.calculate_membership(&seeds, &build_data);
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let mut neighbors = 0;
                let pos = Point::new(x, y);

                let left = pos + Point::new(-1, 0);
                let right = pos + Point::new(1, 0);
                let above = pos + Point::new(0, -1);
                let below = pos + Point::new(0, 1);

                let my_idx = build_data.map.point2d_to_index(pos);
                let my_seed = membership[my_idx];
                if membership[build_data.map.point2d_to_index(left)] != my_seed {
                    neighbors += 1
                }
                if membership[build_data.map.point2d_to_index(right)] != my_seed {
                    neighbors += 1
                }
                if membership[build_data.map.point2d_to_index(above)] != my_seed {
                    neighbors += 1
                }
                if membership[build_data.map.point2d_to_index(below)] != my_seed {
                    neighbors += 1
                }

                if neighbors < 2 {
                    build_data.map.tiles[my_idx] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        }
    }

    fn generate_seeds(
        &self,
        width: i32,
        height: i32,
        rng: &mut RandomNumberGenerator,
        map: &Map,
    ) -> Vec<(usize, Point)> {
        let mut voronoi_seeds: Vec<(usize, Point)> = Vec::new();
        while voronoi_seeds.len() < self.num_seeds {
            let vx = rng.roll_dice(1, width - 1);
            let vy = rng.roll_dice(1, height - 1);
            let pos = Point::new(vx, vy);
            let vidx = map.point2d_to_index(pos);
            let candidate = (vidx, pos);
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }
        voronoi_seeds
    }

    fn calculate_membership(
        &self,
        seeds: &Vec<(usize, Point)>,
        build_data: &BuilderMap,
    ) -> Vec<i32> {
        let mut voronoi_distance = vec![(0, 0.0f32); self.num_seeds];
        let mut voronoi_membership: Vec<i32> =
            vec![0; build_data.map.width * build_data.map.height];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = (i % build_data.map.width) as i32;
            let y = (i / build_data.map.height) as i32;

            for (seed, pos) in seeds.iter().enumerate() {
                let distance = self.compute_distance(Point::new(x, y), pos.1);
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            *vid = voronoi_distance[0].0 as i32;
        }

        voronoi_membership
    }

    fn compute_distance(&self, from: Point, to: Point) -> f32 {
        match self.algorithm {
            DistanceAlgorithm::Pythagoras => DistanceAlg::PythagorasSquared.distance2d(from, to),
            DistanceAlgorithm::Manhattan => DistanceAlg::Manhattan.distance2d(from, to),
            DistanceAlgorithm::Chebyshev => DistanceAlg::Chebyshev.distance2d(from, to),
        }
    }
}
