use crate::prelude::*;

#[system]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn map_render(
    ecs: &SubWorld,
    #[resource] map: &Map,
    #[resource] camera: &Camera,
    #[resource] theme: &MapTheme,
) {
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let player_fov = fov.iter(ecs).nth(0).unwrap();
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);
    for y in camera.top_y..=camera.bottom_y {
        for x in camera.left_x..=camera.right_x {
            let pt = Point::new(x, y);
            let offset = Point::new(camera.left_x, camera.top_y);
            let idx = map_idx(x, y);

            if map.in_bounds(pt)
                && (player_fov.visible_tiles.contains(&pt) | map.revealed_tiles[idx])
            {
                let tint = if player_fov.visible_tiles.contains(&pt) {
                    if map.tiles[idx] == TileType::Wall || map.tiles[idx] == TileType::Floor {
                        PALEGREEN1
                    } else {
                        WHITE
                    }
                } else {
                    DARK_GRAY
                };

                let bg = if map.bloodstains.contains(&idx) {
                    RGB::from_f32(0.75, 0.0, 0.0)
                } else {
                    RGB::named(BLACK)
                };
                let glyph = theme.tile_to_render(map.tiles[idx], map, idx);
                draw_batch.set(pt - offset, ColorPair::new(tint, bg), glyph);
            }
        }
    }

    draw_batch.submit(0).expect("Batch error");
}
