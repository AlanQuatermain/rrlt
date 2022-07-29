use crate::{prelude::*, KeyState};

const ALLOW_SHOW_COORDINATES: bool = false;

#[derive(Clone, Debug)]
struct Tooltip {
    lines: Vec<String>,
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    fn add<S: ToString>(&mut self, line: S) {
        self.lines.push(line.to_string());
    }

    fn width(&self) -> i32 {
        self.lines.iter().map(|s| s.len() as i32).max().unwrap_or(0) + 2i32
    }

    fn height(&self) -> i32 {
        self.lines.len() as i32 + 2i32
    }

    fn render(&self, batch: &mut DrawBatch, pos: Point) {
        let box_color = ColorPair::new(WHITE, GRAY60);
        let title_color = ColorPair::new(WHITE, BLACK);
        let text_color = ColorPair::new(GRAY87, BLACK);
        batch.draw_box(
            Rect::with_size(pos.x, pos.y, self.width() - 1, self.height() - 1),
            box_color,
        );
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { title_color } else { text_color };
            batch.print_color(pos + Point::new(1, i + 1), &s, col);
        }
    }
}

#[system]
#[read_component(Point)]
#[read_component(Name)]
#[read_component(Pools)]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(Hidden)]
#[read_component(Attributes)]
#[read_component(MagicItem)]
#[read_component(ObfuscatedName)]
pub fn tooltips(
    ecs: &SubWorld,
    #[resource] key_state: &KeyState,
    #[resource] camera: &Camera,
    #[resource] map: &Map,
    #[resource] dm: &MasterDungeonMap,
) {
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());

    let offset = Point::new(camera.left_x, camera.top_y);
    let map_pos = key_state.mouse_pos + offset;
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    // Don't show tooltips on things outside the player's view
    if !player_fov.visible_tiles.contains(&map_pos) {
        return;
    }

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    if map_pos.x >= map.width as i32 - 1
        || map_pos.y >= map.height as i32 - 1
        || map_pos.x < 1
        || map_pos.y < 1
    {
        return;
    }

    let mid_point = camera.center_point();
    // println!("Center point: {:?}", mid_point);

    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    <(Entity, &Point)>::query()
        .filter(!component::<Hidden>())
        .iter(ecs)
        .filter(|(_, pos)| **pos == map_pos)
        .filter_map(|(e, _)| {
            if let Ok(entry) = ecs.entry_ref(*e) {
                Some((entry, get_item_display_name(ecs, *e, dm)))
            } else {
                None
            }
        })
        .for_each(|(entry, name)| {
            let mut tip = Tooltip::new();
            tip.add(name.clone());

            // Comment on attributes
            if let Ok(attr) = entry.get_component::<Attributes>() {
                tip.add(attr_str(attr));
            }

            // Comment on stats
            if let Ok(stats) = entry.get_component::<Pools>() {
                tip.add(format!("Level: {}", stats.level));
            }

            tip_boxes.push(tip);
        });

    if tip_boxes.is_empty() {
        return;
    }

    let color = ColorPair::new(WHITE, GRAY60);

    let arrow;
    let arrow_pos;
    if key_state.mouse_pos.x >= mid_point.x {
        // Render to the left
        arrow = to_cp437('→');
        arrow_pos = key_state.mouse_pos + Point::new(-1, 0);
    } else {
        // Render to the right
        arrow = to_cp437('←');
        arrow_pos = key_state.mouse_pos + Point::new(1, 0);
    }
    draw_batch.set(arrow_pos, color, arrow);

    let total_height: i32 = tip_boxes.iter().map(|t| t.height()).sum();
    let mut y = key_state.mouse_pos.y - (total_height / 2);
    while y + (total_height / 2) > 50 {
        y -= 1;
    }
    y = i32::max(y, 0);

    for tt in tip_boxes.iter() {
        let mut x = if key_state.mouse_pos.x >= mid_point.x {
            key_state.mouse_pos.x - (1 + tt.width())
        } else {
            key_state.mouse_pos.x + 2
        };
        x = i32::max(x, 0);
        tt.render(&mut draw_batch, Point::new(x, y));
        y += tt.height();
    }

    draw_batch.submit(10100).expect("Batch error");
}

fn attr_str(attr: &Attributes) -> String {
    let mut words = Vec::new();
    if attr.might.bonus < 0 {
        words.push("Weak.");
    }
    if attr.might.bonus > 0 {
        words.push("Strong.");
    }
    if attr.quickness.bonus < 0 {
        words.push("Clumsy.")
    }
    if attr.quickness.bonus > 0 {
        words.push("Agile.")
    }
    if attr.fitness.bonus < 0 {
        words.push("Unhealthy.")
    }
    if attr.fitness.bonus > 0 {
        words.push("Healthy.")
    }
    if attr.intelligence.bonus < 0 {
        words.push("Unintelligent.")
    }
    if attr.intelligence.bonus > 0 {
        words.push("Smart.")
    }
    if words.is_empty() {
        "Quite Average".to_string()
    } else {
        words.join(" ").to_string()
    }
}
