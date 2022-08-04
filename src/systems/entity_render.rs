use std::collections::HashSet;

use itertools::Itertools;

use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(Render)]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(AlwaysVisible)]
#[read_component(Hidden)]
#[read_component(TileSize)]
pub fn entity_render(ecs: &SubWorld, #[resource] camera: &Camera, #[resource] map: &Map) {
    let renderables = <(&Point, &Render, Option<&TileSize>, Option<&AlwaysVisible>)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(1);
    let offset = Point::new(camera.left_x, camera.top_y);
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    renderables
        .filter(!component::<Player>() & !component::<Hidden>() & !component::<ParticleLifetime>())
        .iter(ecs)
        .filter(|(pos, _, maybe_size, always_visible)| {
            if always_visible.is_some() {
                true
            } else {
                let mut points = HashSet::new();
                if let Some(size) = maybe_size {
                    // pos is upper-left
                    let rect = Rect::with_size(pos.x, pos.y, size.x, size.y);
                    points.extend(rect.point_set());
                } else {
                    points.insert(**pos);
                }
                player_fov.visible_tiles.intersection(&points).count() != 0
            }
        })
        .sorted_by(|a, b| b.1.render_order.cmp(&a.1.render_order))
        .for_each(|(pos, render, maybe_size, _)| {
            let size = maybe_size.unwrap_or(&TileSize { x: 1, y: 1 });
            let rect = Rect::with_size(pos.x, pos.y, size.x, size.y);
            for loc in rect.point_set().iter() {
                if map.in_bounds(*loc) && player_fov.visible_tiles.contains(loc) {
                    draw_batch.set(*loc - offset, render.color, render.glyph);
                }
            }
        });

    draw_batch.submit(5000).expect("Batch error");

    let (pos, render) = <(&Point, &Render)>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();

    draw_batch = DrawBatch::new();
    draw_batch.set(*pos - offset, render.color, render.glyph);
    draw_batch.submit(6000).expect("Batch error");

    draw_batch = DrawBatch::new();
    <(&Point, &Render)>::query()
        .filter(component::<ParticleLifetime>())
        .for_each(ecs, |(pos, render)| {
            draw_batch.set(*pos - offset, render.color, render.glyph);
        });
    draw_batch.submit(7000).expect("Batch error");
}
