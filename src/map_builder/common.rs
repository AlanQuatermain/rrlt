use std::collections::HashMap;

use crate::prelude::*;

const MAX_MONSTERS: i32 = 4;

#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub enum Symmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
        .add("Longsword", map_depth - 1)
        .add("Tower Shield", map_depth - 1)
        .add("Rations", 10)
        .add("Magic Mapping Scroll", 2)
        .add("Bear Trap", 5)
}

pub fn fill_spawns_for_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    spawns: &mut Vec<(Point, String)>,
) {
    let possible_targets: Vec<Point> = room
        .point_set()
        .iter()
        .map(|p| *p)
        .filter(|pos| {
            let idx = map.point2d_to_index(*pos);
            map.tiles[idx] == TileType::Floor
        })
        .collect();

    fill_spawns_for_region(map, rng, &possible_targets, spawns);
}

pub fn fill_spawns_for_region(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    area: &[Point],
    spawns: &mut Vec<(Point, String)>,
) {
    let spawn_table = room_table(map.depth);
    let mut spawn_points: HashMap<Point, String> = HashMap::new();
    let mut areas: Vec<Point> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let num_spawns = i32::min(
            areas.len() as i32,
            rng.roll_dice(1, MAX_MONSTERS + 3) + (map.depth - 1) - 3,
        );
        if num_spawns == 0 {
            return;
        }

        for _ in 0..num_spawns {
            let array_idx = rng.random_slice_index(areas.as_slice()).unwrap();
            let point = areas[array_idx];
            spawn_points.insert(point, spawn_table.roll(rng));
            areas.remove(array_idx);
        }
    }

    for spawn in spawn_points.iter() {
        spawns.push((*spawn.0, spawn.1.to_string()));
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in i32::min(x1, x2)..=i32::max(x1, x2) {
        if let Some(idx) = map.try_idx(Point::new(x, y)) {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in i32::min(y1, y2)..=i32::max(y1, y2) {
        if let Some(idx) = map.try_idx(Point::new(x, y)) {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn build_corridor(map: &mut Map, rng: &mut RandomNumberGenerator, start: Point, end: Point) {
    if rng.roll_dice(1, 2) == 1 {
        apply_horizontal_tunnel(map, start.x, end.x, start.y);
        apply_vertical_tunnel(map, start.y, end.y, end.x);
    } else {
        apply_vertical_tunnel(map, start.y, end.y, start.x);
        apply_horizontal_tunnel(map, start.x, end.x, end.y);
    }
}

pub fn draw_corridor(map: &mut Map, start: Point, end: Point) {
    let mut x = start.x;
    let mut y = start.y;

    while x != end.x || y != end.y {
        if x < end.x {
            x += 1;
        } else if x > end.x {
            x -= 1;
        } else if y < end.y {
            y += 1;
        } else if y > end.y {
            y -= 1;
        }

        if let Some(idx) = map.try_idx(Point::new(x, y)) {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn paint(map: &mut Map, mode: Symmetry, brush_size: i32, pos: Point) {
    let map_width = map.width as i32;
    let map_height = map.height as i32;

    match mode {
        Symmetry::None => apply_paint(map, brush_size, pos),
        Symmetry::Horizontal => {
            let center_x = map_width / 2;
            if pos.x == center_x {
                apply_paint(map, brush_size, pos);
            } else {
                let dist_x = i32::abs(center_x - pos.x);
                apply_paint(map, brush_size, Point::new(center_x + dist_x, pos.y));
                apply_paint(map, brush_size, Point::new(center_x - dist_x, pos.y));
            }
        }
        Symmetry::Vertical => {
            let center_y = map_height / 2;
            if pos.y == center_y {
                apply_paint(map, brush_size, pos);
            } else {
                let dist_y = i32::abs(center_y - pos.y);
                apply_paint(map, brush_size, Point::new(pos.x, center_y + dist_y));
                apply_paint(map, brush_size, Point::new(pos.x, center_y - dist_y));
            }
        }
        Symmetry::Both => {
            let center_x = map_width / 2;
            let center_y = map_height / 2;
            if pos.x == center_x && pos.y == center_y {
                apply_paint(map, brush_size, pos);
            } else {
                let dist_x = i32::abs(center_x - pos.x);
                apply_paint(map, brush_size, Point::new(center_x + dist_x, pos.y));
                apply_paint(map, brush_size, Point::new(center_x - dist_x, pos.y));
                let dist_y = i32::abs(center_y - pos.y);
                apply_paint(map, brush_size, Point::new(pos.x, center_y + dist_y));
                apply_paint(map, brush_size, Point::new(pos.x, center_y - dist_y));
            }
        }
    }
}

fn apply_paint(map: &mut Map, brush_size: i32, pos: Point) {
    let map_width = map.width as i32;
    let map_height = map.height as i32;

    match brush_size {
        1 => {
            let digger_idx = map.point2d_to_index(pos);
            map.tiles[digger_idx] = TileType::Floor;
        }
        _ => {
            let half_brush_size = brush_size / 2;
            for brush_y in pos.y - half_brush_size..pos.y + half_brush_size {
                for brush_x in pos.x - half_brush_size..pos.x + half_brush_size {
                    if brush_x > 1
                        && brush_x < map_width - 1
                        && brush_y > 1
                        && brush_y < map_height - 1
                    {
                        let idx = map_idx(brush_x, brush_y);
                        map.tiles[idx] = TileType::Floor;
                    }
                }
            }
        }
    }
}
