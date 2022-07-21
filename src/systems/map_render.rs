use crate::prelude::*;

#[system]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn map_render(ecs: &SubWorld, #[resource] map: &Map, #[resource] camera: &Camera) {
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let player_fov = fov.iter(ecs).nth(0).unwrap();
    let offset = Point::new(camera.left_x, camera.top_y);
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);
    for y in camera.top_y..=camera.bottom_y {
        for x in camera.left_x..=camera.right_x {
            let pt = Point::new(x, y);
            if !map.in_bounds(pt) {
                if SHOW_BOUNDARIES {
                    draw_batch.set(pt - offset, ColorPair::new(GREY, BLACK), to_cp437('.'));
                }
                continue;
            }
            let idx = map.point2d_to_index(pt);

            if player_fov.visible_tiles.contains(&pt) | map.revealed_tiles[idx] {
                let bg = if map.bloodstains.contains(&idx) {
                    RGB::from_f32(0.75, 0.0, 0.0)
                } else {
                    RGB::named(BLACK)
                };
                let (glyph, mut fg) = map.theme.tile_to_render(map, idx);
                if !player_fov.visible_tiles.contains(&pt) && !map.visible_tiles[idx] {
                    fg = fg.to_greyscale();
                }
                draw_batch.set(pt - offset, ColorPair::new(fg, bg), glyph);
            }
        }
    }

    draw_batch.submit(0).expect("Batch error");
}
