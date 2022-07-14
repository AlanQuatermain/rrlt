use crate::prelude::*;

#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub enum Symmetry { None, Horizontal, Vertical, Both }

pub fn paint(map: &mut Map, mode: Symmetry, brush_size: i32, pos: Point) {
    const MAP_WIDTH: i32 = super::MAP_WIDTH as i32;
    const MAP_HEIGHT: i32 = super::MAP_HEIGHT as i32;

    match mode {
        Symmetry::None => apply_paint(map, brush_size, pos),
        Symmetry::Horizontal => {
            let center_x = MAP_WIDTH / 2;
            if pos.x == center_x {
                apply_paint(map, brush_size, pos);
            }
            else {
                let dist_x = i32::abs(center_x - pos.x);
                apply_paint(map, brush_size, Point::new(center_x + dist_x, pos.y));
                apply_paint(map, brush_size, Point::new(center_x - dist_x, pos.y));
            }
        }
        Symmetry::Vertical => {
            let center_y = MAP_HEIGHT / 2;
            if pos.y == center_y {
                apply_paint(map, brush_size, pos);
            }
            else {
                let dist_y = i32::abs(center_y - pos.y);
                apply_paint(map, brush_size, Point::new(pos.x, center_y + dist_y));
                apply_paint(map, brush_size,Point::new(pos.x, center_y - dist_y));
            }
        }
        Symmetry::Both => {
            let center_x = MAP_WIDTH / 2;
            let center_y = MAP_HEIGHT / 2;
            if pos.x == center_x && pos.y == center_y {
                apply_paint(map, brush_size, pos);
            }
            else {
                let dist_x = i32::abs(center_x - pos.x);
                apply_paint(map, brush_size, Point::new(center_x + dist_x, pos.y));
                apply_paint(map, brush_size, Point::new(center_x - dist_x, pos.y));
                let dist_y = i32::abs(center_y - pos.y);
                apply_paint(map, brush_size, Point::new(pos.x, center_y + dist_y));
                apply_paint(map, brush_size,Point::new(pos.x, center_y - dist_y));
            }
        }
    }
}

fn apply_paint(map: &mut Map, brush_size: i32, pos: Point) {
    const MAP_WIDTH: i32 = super::MAP_WIDTH as i32;
    const MAP_HEIGHT: i32 = super::MAP_HEIGHT as i32;

    match brush_size {
        1 => {
            let digger_idx = map.point2d_to_index(pos);
            map.tiles[digger_idx] = TileType::Floor;
        }
        _ => {
            let half_brush_size = brush_size/2;
            for brush_y in pos.y-half_brush_size .. pos.y+half_brush_size {
                for brush_x in pos.x-half_brush_size .. pos.x+half_brush_size {
                    if brush_x > 1 && brush_x < MAP_WIDTH-1 && brush_y > 1 && brush_y < MAP_HEIGHT-1 {
                        let idx = map_idx(brush_x, brush_y);
                        map.tiles[idx] = TileType::Floor;
                    }
                }
            }
        }
    }
}