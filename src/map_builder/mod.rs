mod empty;
mod rooms;
mod themes;
mod bsp;

use crate::prelude::*;
// use empty::EmptyArchitect;
use rooms::RoomsArchitect;
// use themes::*;

pub use themes::MapTheme;
use crate::map_builder::bsp::BSPArchitect;

const MAX_ROOMS: usize = 30;
const MIN_SIZE: usize = 6;
const MAX_SIZE: usize = 10;

const MAX_SPAWNS_PER_ROOM: i32 = 4;

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator, depth: i32) -> MapBuilder;
}

pub struct MapBuilder {
    pub map: Map,
    pub depth: i32,
    pub rooms: Vec<Rect>,
    pub spawns: Vec<Point>,
    pub player_start: Point,
    pub goal_start: Point,
    pub theme: MapTheme,
    pub random_table: RandomTable,

    pub history: Vec<Map>
}
impl Default for MapBuilder {
    fn default() -> Self {
        MapBuilder{
            map: Map::new(),
            depth: 0,
            rooms: Vec::default(),
            spawns: Vec::default(),
            player_start: Point::zero(),
            goal_start: Point::zero(),
            theme: MapTheme::default(),
            random_table: RandomTable::default(),
            history: Vec::default()
        }
    }
}

impl MapBuilder {
    pub fn new(rng: &mut RandomNumberGenerator, depth: i32) -> Self {
        let mut architect: Box<dyn MapArchitect> = match rng.roll_dice(1, 2) {
            1 => Box::new(RoomsArchitect{}),
            _ => Box::new(BSPArchitect::default()),
        };
        let mut mb = architect.new(rng, depth);

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
            1024.0
        );

        const UNREACHABLE: &f32 = &f32::MAX;
        self.map.index_to_point2d(
            dmap.map
                .iter()
                .enumerate()
                .filter(|(_, dist)| *dist < UNREACHABLE)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap().0
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
                h as usize
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
        use std::cmp::{min, max};
        for y in min(y1, y2) ..= max(y1, y2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        use std::cmp::{min, max};
        for x in min(x1, x2) ..= max(x1, x2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn build_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        let mut rooms = self.rooms.clone();
        rooms.sort_by(|a, b| a.center().x.cmp(&b.center().x));

        for (i, room) in rooms.iter().enumerate().skip(1) {
            let prev = rooms[i-1].center();
            let new = room.center();
            self.build_corridor(prev, new, rng);
            self.take_snapshot();
        }
    }

    fn build_corridor(&mut self, start: Point, end: Point, rng: &mut RandomNumberGenerator) {
        if rng.range(0, 2) == 1 {
            self.apply_horizontal_tunnel(start.x, end.x, start.y);
            self.apply_vertical_tunnel(start.y, end.y, end.x);
        }
        else {
            self.apply_vertical_tunnel(start.y, end.y, start.x);
            self.apply_horizontal_tunnel(start.x, end.x, end.y);
        }
    }

    fn spawn_room(&mut self, room: &Rect, rng: &mut RandomNumberGenerator) {
        let num_spawns = rng.roll_dice(1, MAX_SPAWNS_PER_ROOM + 3) + self.depth - 3;
        let mut spawnable_tiles = Vec::from_iter(room.point_set());

        for _ in 0 .. num_spawns {
            if spawnable_tiles.is_empty() {
                break;
            }
            let target_index = rng.random_slice_index(&spawnable_tiles)
                .unwrap();
            self.spawns.push(spawnable_tiles[target_index].clone());
            spawnable_tiles.remove(target_index);
        }
    }

    fn _spawn_monsters(
        &self,
        start: &Point,
        rng: &mut RandomNumberGenerator
    ) -> Vec<Point> {
        const NUM_MONSTERS: usize = 50;
        let mut spawnable_tiles: Vec<Point> = self.map.tiles
            .iter()
            .enumerate()
            .filter(|(idx, t)| {
                **t == TileType::Floor && DistanceAlg::Pythagoras.distance2d(
                    *start,
                    self.map.index_to_point2d(*idx)
                ) > 10.0
            })
            .map(|(idx, _)| self.map.index_to_point2d(idx))
            .collect();

        let mut spawns = Vec::new();
        for _ in 0 .. NUM_MONSTERS {
            let target_index = rng.random_slice_index(&spawnable_tiles)
                .unwrap();
            spawns.push(spawnable_tiles[target_index].clone());
            spawnable_tiles.remove(target_index);
        }

        spawns
    }
}
