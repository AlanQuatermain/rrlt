use crate::prelude::*;

const ALLOW_SHOW_COORDINATES: bool = false;

#[system]
#[read_component(Point)]
#[read_component(Name)]
#[read_component(Pools)]
#[read_component(Armor)]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn tooltips(
    ecs: &SubWorld,
    #[resource] mouse_pos: &Point,
    #[resource] camera: &Camera,
    #[resource] key: &Option<VirtualKeyCode>,
) {
    let positions = <(Entity, &Point, &Name)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());

    let offset = Point::new(camera.left_x, camera.top_y);
    let map_pos = *mouse_pos + offset;
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    let mut tooltips: Vec<String> = Vec::new();
    positions
        .filter(!component::<Hidden>())
        .iter(ecs)
        .filter(|(_, pos, _)| **pos == map_pos && player_fov.visible_tiles.contains(&pos))
        .for_each(|(entity, _, name)| {
            let display =
                if let Ok(stats) = ecs.entry_ref(*entity).unwrap().get_component::<Pools>() {
                    format!("{} : {} hp", &name.0, stats.hit_points.current)
                } else {
                    name.0.clone()
                };
            tooltips.push(display);
        });

    if ALLOW_SHOW_COORDINATES {
        tooltips.push(format!("map: {},{}", map_pos.x, map_pos.y));
        tooltips.push(format!("src: {},{}", mouse_pos.x, mouse_pos.y));
    }

    if tooltips.is_empty() {
        return;
    }

    let mut width: i32 = 0;
    for s in tooltips.iter() {
        if width < s.len() as i32 {
            width = s.len() as i32
        }
    }
    width += 3;

    if mouse_pos.x > 40 {
        let arrow_pos = *mouse_pos - Point::new(2, 0);
        let left_x = mouse_pos.x - width;
        let mut y = mouse_pos.y;
        for s in tooltips.iter() {
            draw_batch.print_color(Point::new(left_x, y), s, ColorPair::new(WHITE, GREY));
            let padding = (width - s.len() as i32) - 1;
            for i in 0..padding {
                draw_batch.print_color(
                    Point::new(arrow_pos.x - i, y),
                    &" ".to_string(),
                    ColorPair::new(WHITE, GREY),
                );
            }
            y += 1;
        }
        draw_batch.print_color(arrow_pos, &"->".to_string(), ColorPair::new(WHITE, GREY));
    } else {
        let arrow_pos = *mouse_pos + Point::new(1, 0);
        let left_x = mouse_pos.x + 3;
        let mut y = mouse_pos.y;
        for s in tooltips.iter() {
            draw_batch.print_color(Point::new(left_x + 1, y), s, ColorPair::new(WHITE, GREY));
            let padding = (width - s.len() as i32) - 1;
            for i in 0..padding {
                draw_batch.print_color(
                    Point::new(left_x + 1 + s.len() as i32 + i, y),
                    &" ".to_string(),
                    ColorPair::new(WHITE, GREY),
                );
            }
            y += 1;
        }
        draw_batch.print_color(arrow_pos, &"<- ".to_string(), ColorPair::new(WHITE, GREY));
    }
    draw_batch.submit(10100).expect("Batch error");
}
