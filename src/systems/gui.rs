use crate::prelude::*;

#[system]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(Pools)]
#[read_component(Attributes)]
#[read_component(HungerClock)]
#[read_component(Equipped)]
#[read_component(Consumable)]
#[read_component(Carried)]
pub fn gui(ecs: &SubWorld, #[resource] gamelog: &Gamelog, #[resource] map: &Map) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    let box_color = ColorPair::new(GRAY60, BLACK);
    let text_color = ColorPair::new(WHITE, BLACK);
    let hp_color = ColorPair::new(RED, BLACK);
    let mana_color = ColorPair::new(BLUE, BLACK);

    // Layout boxes
    draw_batch.draw_hollow_box(Rect::with_size(0, 0, 79, 59), box_color);
    draw_batch.draw_hollow_box(Rect::with_size(0, 0, 49, 45), box_color);
    draw_batch.draw_hollow_box(Rect::with_size(0, 45, 79, 14), box_color);
    draw_batch.draw_hollow_box(Rect::with_size(49, 0, 30, 8), box_color);

    // Put in some connectors to join things up.
    draw_batch.set(Point::new(0, 45), box_color, to_cp437('├'));
    draw_batch.set(Point::new(49, 8), box_color, to_cp437('├'));
    draw_batch.set(Point::new(49, 0), box_color, to_cp437('┬'));
    draw_batch.set(Point::new(49, 45), box_color, to_cp437('┴'));
    draw_batch.set(Point::new(79, 8), box_color, to_cp437('┤'));
    draw_batch.set(Point::new(79, 45), box_color, to_cp437('┤'));

    // Map title
    let name_len = map.name.len() + 2;
    let x_pos = (22 - (name_len / 2)) as i32;
    draw_batch.set(Point::new(x_pos, 0), box_color, to_cp437('┤'));
    draw_batch.set(
        Point::new(x_pos + name_len as i32 - 1, 0),
        box_color,
        to_cp437('├'),
    );
    draw_batch.print_color(Point::new(x_pos + 1, 0), &map.name, text_color);

    // Get the player entity.
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();
    let player = ecs.entry_ref(*player_entity).unwrap();

    // Stats
    if let Ok(stats) = player.get_component::<Pools>() {
        let health = format!(
            "Health: {}/{}",
            stats.hit_points.current, stats.hit_points.max
        );
        let mana = format!("Mana:   {}/{}", stats.mana.current, stats.mana.max);
        draw_batch.print_color(Point::new(50, 1), health, text_color);
        draw_batch.print_color(Point::new(50, 2), mana, text_color);
        draw_batch.bar_horizontal(
            Point::new(64, 1),
            14,
            stats.hit_points.current,
            stats.hit_points.max,
            hp_color,
        );
        draw_batch.bar_horizontal(
            Point::new(64, 2),
            14,
            stats.mana.current,
            stats.mana.max,
            mana_color,
        );
    }

    // Attributes
    if let Ok(attrs) = player.get_component::<Attributes>() {
        draw_attribute("Might:", &attrs.might, 4, &mut draw_batch);
        draw_attribute("Quickness:", &attrs.might, 5, &mut draw_batch);
        draw_attribute("Fitness:", &attrs.might, 6, &mut draw_batch);
        draw_attribute("Intelligence:", &attrs.might, 7, &mut draw_batch);
    }

    // Equipped items
    let mut y = 9;
    <(&Name, &Equipped)>::query()
        .iter(ecs)
        .filter(|(_, e)| e.owner == *player_entity)
        .for_each(|(name, _)| {
            draw_batch.print_color(Point::new(50, y), &name.0, text_color);
            y += 1;
        });

    // Consumables
    y += 1;
    let yellow = ColorPair::new(YELLOW, BLACK);
    let green = ColorPair::new(GREEN, BLACK);
    let mut index = 1;
    <(&Carried, &Name)>::query()
        .filter(component::<Consumable>())
        .iter(ecs)
        .filter(|(c, _)| c.0 == *player_entity)
        .for_each(|(_, name)| {
            if index < 10 {
                draw_batch.print_color(Point::new(50, y), &format!("↑{}", index), yellow);
                draw_batch.print_color(Point::new(53, y), &name.0, green);
                y += 1;
                index += 1;
            }
        });

    // Status
    let orange = ColorPair::new(ORANGE, BLACK);
    let red = ColorPair::new(RED, BLACK);
    if let Ok(hunger) = player.get_component::<HungerClock>() {
        let pos = Point::new(50, 44);
        match hunger.state {
            HungerState::WellFed => {
                draw_batch.print_color(pos, "Well Fed", green);
            }
            HungerState::Normal => {}
            HungerState::Hungry => {
                draw_batch.print_color(pos, "Hungry", orange);
            }
            HungerState::Starving => {
                draw_batch.print_color(pos, "Starving", red);
            }
        }
    }

    // Logs
    let mut y = 46;
    for s in gamelog.entries.iter().rev() {
        if y < 59 {
            draw_batch.print(Point::new(2, y), s);
            y += 1;
        }
    }

    draw_batch.submit(9999).expect("Batch error");
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, batch: &mut DrawBatch) {
    let name_color = ColorPair::new(GRAY80, BLACK);
    let pos_color = ColorPair::new(GREEN, BLACK);
    let mid_color = ColorPair::new(WHITE, BLACK);

    batch.print_color(Point::new(50, y), name, name_color);
    let color = if attribute.modifiers < 0 {
        ColorPair::new(RED, BLACK)
    } else if attribute.modifiers == 0 {
        ColorPair::new(WHITE, BLACK)
    } else {
        ColorPair::new(GREEN, BLACK)
    };
    batch.print_color(
        Point::new(67, y),
        &format!("{}", attribute.base + attribute.modifiers),
        color,
    );
    batch.print_color(Point::new(73, y), format!("{}", attribute.bonus), color);

    if attribute.bonus > 0 {
        batch.set(Point::new(72, y), color, to_cp437('+'));
    }
}

#[system]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(Pools)]
#[read_component(HungerClock)]
fn old_gui(ecs: &SubWorld, #[resource] gamelog: &Gamelog, #[resource] map: &Map) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    draw_batch.draw_box(Rect::with_size(0, 43, 79, 6), ColorPair::new(WHITE, BLACK));

    let player = <&Player>::query().iter(ecs).nth(0).unwrap();
    let depth = format!("Depth: {}", player.map_level + 1);
    draw_batch.print_color(Point::new(2, 43), depth, ColorPair::new(YELLOW, BLACK));

    let mut health_query = <(&Pools, &HungerClock)>::query().filter(component::<Player>());
    let (stats, hunger_clock) = health_query.iter(ecs).nth(0).unwrap();
    let health = format!(
        " HP: {} / {}",
        stats.hit_points.current, stats.hit_points.max
    );
    draw_batch.print_color(Point::new(12, 43), health, ColorPair::new(YELLOW, BLACK));
    draw_batch.bar_horizontal(
        Point::new(28, 43),
        51,
        stats.hit_points.current,
        stats.hit_points.max,
        ColorPair::new(RED, BLACK),
    );

    match hunger_clock.state {
        HungerState::WellFed => {
            draw_batch.print_color_right(
                Point::new(71, 42),
                "Well Fed",
                ColorPair::new(GREEN, BLACK),
            );
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            draw_batch.print_color_right(
                Point::new(71, 42),
                "Hungry",
                ColorPair::new(ORANGE, BLACK),
            );
        }
        HungerState::Starving => {
            draw_batch.print_color_right(
                Point::new(71, 42),
                "Starving",
                ColorPair::new(RED, BLACK),
            );
        }
    }

    let mut y = 44;
    for s in gamelog.entries.iter().rev() {
        if y < 49 {
            draw_batch.print(Point::new(2, y), s);
        }
        y += 1;
    }

    draw_batch.submit(5000).expect("Batch error");
}
