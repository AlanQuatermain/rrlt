mod automata;
mod bsp;
mod bsp_interior;
mod common;
mod dla;
mod drunkard;
mod empty;
mod maze;
mod prefab;
mod rooms;
mod themes;
mod voronoi;
mod waveform_collapse;

use crate::prelude::*;
use std::collections::HashMap;
// use empty::EmptyArchitect;
use rooms::RoomsArchitect;
// use themes::*;

use automata::CellularAutomataArchitect;
use bsp::BSPArchitect;
use bsp_interior::BSPInteriorArchitect;
use dla::DLAArchitect;
use drunkard::DrunkardsWalkArchitect;
use maze::MazeArchitect;
use prefab::PrefabArchitect;
pub use themes::MapTheme;
use voronoi::VoronoiArchitect;
use waveform_collapse::WaveformCollapseArchitect;

const MAX_ROOMS: usize = 30;
const MIN_SIZE: usize = 6;
const MAX_SIZE: usize = 10;

const MAX_SPAWNS_PER_ROOM: i32 = 4;

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder;
    fn spawn(&mut self, ecs: &mut World, mb: &mut MapBuilder, rng: &mut RandomNumberGenerator);
}

#[derive(Clone, PartialEq)]
pub struct MapBuilder {
    pub map: Map,
    pub depth: i32,
    pub rooms: Vec<Rect>,
    pub spawns: Vec<Point>,
    pub player_start: Point,
    pub goal_start: Point,
    pub theme: MapTheme,
    pub random_table: RandomTable,

    pub history: Vec<Map>,
}
impl Default for MapBuilder {
    fn default() -> Self {
        MapBuilder {
            map: Map::new(),
            depth: 0,
            rooms: Vec::default(),
            spawns: Vec::default(),
            player_start: Point::zero(),
            goal_start: Point::zero(),
            theme: MapTheme::default(),
            random_table: RandomTable::default(),
            history: Vec::default(),
        }
    }
}

fn random_architect(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    depth: i32,
) -> Box<dyn MapArchitect> {
    let mut architect: Box<dyn MapArchitect> = match rng.roll_dice(1, 17) {
        // 1 => Box::new(RoomsArchitect::default()),
        // 2 => Box::new(BSPArchitect::default()),
        // 3 => Box::new(BSPInteriorArchitect::default()),
        // 4 => Box::new(CellularAutomataArchitect::default()),
        // 5 => Box::new(DrunkardsWalkArchitect::open_area()),
        // 6 => Box::new(DrunkardsWalkArchitect::open_halls()),
        // 7 => Box::new(DrunkardsWalkArchitect::winding_passages()),
        // 8 => Box::new(DrunkardsWalkArchitect::fat_passages()),
        // 9 => Box::new(DrunkardsWalkArchitect::fearful_symmetry()),
        // 10 => Box::new(MazeArchitect::default()),
        // 11 => Box::new(DLAArchitect::walk_inwards()),
        // 12 => Box::new(DLAArchitect::walk_outwards()),
        // 13 => Box::new(DLAArchitect::central_attractor()),
        // 14 => Box::new(DLAArchitect::rorschach()),
        // 15 => Box::new(VoronoiArchitect::pythagoras()),
        // 16 => Box::new(VoronoiArchitect::manhattan()),
        // _ => Box::new(VoronoiArchitect::chebyshev()),
        // _ => Box::new(PrefabArchitect::default()),
        _ => {
            let mut base_architect = CellularAutomataArchitect::default();
            let mut mb = base_architect.new(rng, depth);
            base_architect.spawn(ecs, &mut mb, rng);
            Box::new(PrefabArchitect::sectional(prefab::UNDERGROUND_FORT, &mb))
        }
    };

    // if rng.roll_dice(1, 3) == 1 {
    //     architect = Box::new(WaveformCollapseArchitect::derived_map(architect));
    // }

    architect
}

impl MapBuilder {
    pub fn new(ecs: &mut World, rng: &mut RandomNumberGenerator, depth: i32) -> Self {
        let mut architect = random_architect(ecs, rng, depth);
        let mut mb = architect.new(rng, depth);
        architect.spawn(ecs, &mut mb, rng);

        mb.theme = match rng.range(0, 2) {
            _ => MapTheme::Dungeon,
        };

        mb
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            snapshot.revealed_tiles.iter_mut().for_each(|v| *v = true);
            self.history.push(snapshot);
        }
    }

    fn generate_random_table(&mut self) {
        self.random_table = RandomTable::new()
            .add("Goblin", 10)
            .add("Orc", 1 + self.depth)
            .add("Ogre", self.depth - 2)
            .add("Ettin", self.depth - 4)
            .add("Health Potion", 7)
            .add("Fireball Scroll", 2 + self.depth)
            .add("Confusion Scroll", 2 + self.depth)
            .add("Magic Missile Scroll", 4)
            .add("Dungeon Map", 2)
            .add("Dagger", 3)
            .add("Shield", 3)
            .add("Longsword", self.depth - 1)
            .add("Tower Shield", self.depth - 1)
            .add("Bear Trap", 2)
            .add("Rations", 10);
    }

    fn fill(&mut self, tile: TileType) {
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
    }

    fn find_most_distant(&self) -> Point {
        let dmap = DijkstraMap::new(
            MAP_WIDTH,
            MAP_HEIGHT,
            &vec![self.map.point2d_to_index(self.player_start)],
            &self.map,
            1024.0,
        );

        const UNREACHABLE: &f32 = &f32::MAX;
        self.map.index_to_point2d(
            dmap.map
                .iter()
                .enumerate()
                .filter(|(_, dist)| *dist < UNREACHABLE)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0,
        )
    }

    fn build_random_rooms(&mut self, rng: &mut RandomNumberGenerator) {
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let room = Rect::with_size(
                rng.range(1, MAP_WIDTH - w - 1),
                rng.range(1, MAP_HEIGHT - h - 1),
                w as usize,
                h as usize,
            );
            let mut overlap = false;
            for r in self.rooms.iter() {
                if r.intersect(&room) {
                    overlap = true;
                }
            }
            if !overlap {
                room.for_each(|p| {
                    if p.x > 0 && p.x < SCREEN_WIDTH && p.y > 0 && p.y < SCREEN_HEIGHT {
                        let idx = map_idx(p.x, p.y);
                        self.map.tiles[idx] = TileType::Floor;
                    }
                });

                self.rooms.push(room);
                self.take_snapshot();
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        use std::cmp::{max, min};
        for y in min(y1, y2)..=max(y1, y2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        use std::cmp::{max, min};
        for x in min(x1, x2)..=max(x1, x2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn build_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        let mut rooms = self.rooms.clone();
        rooms.sort_by(|a, b| a.center().x.cmp(&b.center().x));

        for (i, room) in rooms.iter().enumerate().skip(1) {
            let prev = rooms[i - 1].center();
            let new = room.center();
            self.build_corridor(prev, new, rng);
            self.take_snapshot();
        }
    }

    fn build_corridor(&mut self, start: Point, end: Point, rng: &mut RandomNumberGenerator) {
        if rng.range(0, 2) == 1 {
            self.apply_horizontal_tunnel(start.x, end.x, start.y);
            self.apply_vertical_tunnel(start.y, end.y, end.x);
        } else {
            self.apply_vertical_tunnel(start.y, end.y, start.x);
            self.apply_horizontal_tunnel(start.x, end.x, end.y);
        }
    }

    fn spawn_room(&mut self, room: &Rect, rng: &mut RandomNumberGenerator) {
        let num_spawns = rng.roll_dice(1, MAX_SPAWNS_PER_ROOM + 3) + self.depth - 3;
        let mut spawnable_tiles = Vec::from_iter(room.point_set());

        for _ in 0..num_spawns {
            if spawnable_tiles.is_empty() {
                break;
            }
            let target_index = rng.random_slice_index(&spawnable_tiles).unwrap();
            self.spawns.push(spawnable_tiles[target_index].clone());
            spawnable_tiles.remove(target_index);
        }
    }

    #[allow(dead_code)]
    fn spawn_entities(&self, start: &Point, rng: &mut RandomNumberGenerator) -> Vec<Point> {
        const NUM_MONSTERS: usize = 50;
        let mut spawnable_tiles: Vec<Point> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(idx, t)| {
                **t == TileType::Floor
                    && DistanceAlg::Pythagoras.distance2d(*start, self.map.index_to_point2d(*idx))
                        > 10.0
            })
            .map(|(idx, _)| self.map.index_to_point2d(idx))
            .collect();

        let mut spawns = Vec::new();
        for _ in 0..NUM_MONSTERS {
            let target_index = rng.random_slice_index(&spawnable_tiles).unwrap();
            spawns.push(spawnable_tiles[target_index].clone());
            spawnable_tiles.remove(target_index);
        }

        spawns
    }

    fn spawn_voronoi_regions(&mut self, rng: &mut RandomNumberGenerator) {
        let mut noise_areas: HashMap<i32, Vec<usize>> = HashMap::new();
        let mut noise = FastNoise::seeded(rng.next_u64());
        noise.set_noise_type(NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(CellularDistanceFunction::Manhattan);

        for y in 1..MAP_HEIGHT as i32 - 1 {
            for x in 1..MAP_WIDTH as i32 - 1 {
                let idx = map_idx(x, y);
                if self.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if noise_areas.contains_key(&cell_value) {
                        noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }

        for area in noise_areas.iter() {
            let num_spawns = i32::min(
                area.1.len() as i32,
                rng.roll_dice(1, MAX_SPAWNS_PER_ROOM + 3) + (self.depth) - 3,
            );
            if num_spawns == 0 {
                continue;
            }

            let mut values = area.1.clone();
            for _ in 0..num_spawns {
                let idx = rng.random_slice_index(values.as_slice()).unwrap();
                self.spawns.push(self.map.index_to_point2d(values[idx]));
                values.remove(idx);
            }
        }
    }

    fn prune_unreachable_regions(&mut self, start: Point) {
        let start_idx = self.map.point2d_to_index(start);
        let dijkstra_map =
            DijkstraMap::new(MAP_WIDTH, MAP_HEIGHT, &vec![start_idx], &self.map, 1024.0);
        dijkstra_map
            .map
            .iter()
            .enumerate()
            .filter(|(_, distance)| *distance > &2000.0)
            .for_each(|(idx, _)| self.map.tiles[idx] = TileType::Wall);
    }
}
