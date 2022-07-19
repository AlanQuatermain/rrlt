use itertools::Itertools;

use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(Render)]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(AlwaysVisible)]
#[read_component(Hidden)]
pub fn entity_render(ecs: &SubWorld, #[resource] camera: &Camera) {
    let renderables = <(&Point, &Render, Entity)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(1);
    let offset = Point::new(camera.left_x, camera.top_y);
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    renderables
        .filter(!component::<Player>() & !component::<Hidden>())
        .iter(ecs)
        .filter(|(pos, _, entity)| {
            ecs.entry_ref(**entity)
                .unwrap()
                .get_component::<AlwaysVisible>()
                .is_ok()
                || player_fov.visible_tiles.contains(&pos)
        })
        .sorted_by(|a, b| b.1.render_order.cmp(&a.1.render_order))
        .for_each(|(pos, render, _)| {
            draw_batch.set(*pos - offset, render.color, render.glyph);
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
}
