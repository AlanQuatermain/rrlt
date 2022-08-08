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
#[read_component(MagicItem)]
#[read_component(ObfuscatedName)]
#[read_component(CursedItem)]
#[read_component(StatusEffect)]
#[read_component(Duration)]
#[read_component(Name)]
#[read_component(KnownSpells)]
#[read_component(Weapon)]
pub fn gui(
    ecs: &SubWorld,
    // #[resource] gamelog: &Gamelog,
    #[resource] map: &Map,
    #[resource] dm: &MasterDungeonMap,
) {
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
        let level = format!("Level:  {}", stats.level);
        let xp_level_start = (stats.level - 1) * 1000;

        draw_batch.print_color(Point::new(50, 1), health, text_color);
        draw_batch.print_color(Point::new(50, 2), mana, text_color);
        draw_batch.print_color(Point::new(50, 3), level, text_color);

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
        draw_batch.bar_horizontal(
            Point::new(64, 3),
            14,
            stats.xp - xp_level_start,
            1000,
            ColorPair::new(GOLD, BLACK),
        );
    }

    // Attributes
    if let Ok(attrs) = player.get_component::<Attributes>() {
        draw_attribute("Might:", &attrs.might, 4, &mut draw_batch);
        draw_attribute("Quickness:", &attrs.quickness, 5, &mut draw_batch);
        draw_attribute("Fitness:", &attrs.fitness, 6, &mut draw_batch);
        draw_attribute("Intelligence:", &attrs.intelligence, 7, &mut draw_batch);

        // Initiative, weight, and gold
        if let Ok(stats) = player.get_component::<Pools>() {
            draw_batch.print_color(
                Point::new(50, 9),
                &format!(
                    "{:.1} lbs ({} lbs max)",
                    stats.total_weight,
                    attrs.max_weight()
                ),
                text_color,
            );
            draw_batch.print_color(
                Point::new(50, 10),
                &format!("Initiative Penalty: {:.0}", stats.total_initiative_penalty),
                text_color,
            );
            draw_batch.print_color(
                Point::new(50, 11),
                &format!("Gold: {:.1}", stats.gold),
                ColorPair::new(GOLD, BLACK),
            );
        }
    }

    // Equipped items
    let mut y = 13;
    <(&Equipped, Entity, Option<&Weapon>)>::query()
        .iter(ecs)
        .filter(|(e, _, _)| e.owner == *player_entity)
        .for_each(|(_, item, wpn)| {
            let name = get_item_display_name(ecs, *item, dm);
            draw_batch.print_color(
                Point::new(50, y),
                truncate(name.clone(), 25),
                get_item_color(ecs, *item, dm),
            );
            y += 1;

            if let Some(weapon) = wpn {
                let mut weapon_info = format!("┤ {} ({})", &name, weapon.damage_die);
                if let Some(range) = weapon.range {
                    weapon_info += &format!(" (range: {}, F: fire, V: cycle targets)", range);
                }
                weapon_info += " ├";
                draw_batch.print_color(
                    Point::new(3, 45),
                    &weapon_info,
                    ColorPair::new(YELLOW, BLACK),
                );
            }
        });

    // Consumables
    y += 1;
    let yellow = ColorPair::new(YELLOW, BLACK);
    let green = ColorPair::new(GREEN, BLACK);
    let mut index = 1;
    <(&Carried, Entity)>::query()
        .filter(component::<Consumable>())
        .iter(ecs)
        .filter(|(c, _)| c.0 == *player_entity)
        .for_each(|(_, item)| {
            if index < 10 {
                draw_batch.print_color(Point::new(50, y), &format!("↑{}", index), yellow);
                draw_batch.print_color(
                    Point::new(53, y),
                    truncate(get_item_display_name(ecs, *item, dm), 25),
                    get_item_color(ecs, *item, dm),
                );
                y += 1;
                index += 1;
            }
        });

    // Spells
    y += 1;
    let blue = ColorPair::new(CYAN, BLACK);
    if let Ok(known) = player.get_component::<KnownSpells>() {
        let mut index = 1;
        for spell in known.spells.iter() {
            draw_batch.print_color(Point::new(50, y), &format!("^{}", index), blue);
            draw_batch.print_color(
                Point::new(53, y),
                &format!("{} ({})", &spell.display_name, spell.mana_cost),
                blue,
            );
            index += 1;
            y += 1;
        }
    }

    // Status
    let mut y = 44;
    let orange = ColorPair::new(ORANGE, BLACK);
    let red = ColorPair::new(RED, BLACK);
    let hclock = player.get_component::<HungerClock>().unwrap();
    match hclock.state {
        HungerState::WellFed => {
            draw_batch.print_color(Point::new(50, y), "Well Fed", green);
            y -= 1;
        }
        HungerState::Normal => {}
        HungerState::Hungry => {
            draw_batch.print_color(Point::new(50, y), "Hungry", orange);
            y -= 1;
        }
        HungerState::Starving => {
            draw_batch.print_color(Point::new(50, y), "Starving", red);
            y -= 1;
        }
    }
    <(&StatusEffect, &Duration, &Name)>::query()
        .iter(ecs)
        .filter(|(s, _, _)| s.target == *player_entity)
        .for_each(|(_, duration, name)| {
            draw_batch.print_color(
                Point::new(50, y),
                &format!("{} ({})", &name.0, duration.0),
                red,
            );
            y -= 1;
        });

    draw_batch.submit(9998).expect("Batch error");

    // let mut log_batch = DrawBatch::new();
    // log_batch.target(3);

    // Draw the log
    let mut block = TextBlock::new(1, 46 / 2, 79, 58 / 2);
    block
        .print(&crate::gamelog::log_display())
        .expect("Failed to get log contents");
    block.render(&mut BACKEND_INTERNAL.lock().consoles[3].console);

    draw_batch.submit(9999).expect("Batch error");
}

fn truncate(str: String, max_len: usize) -> String {
    if str.len() > max_len {
        let mut newstr = str.clone();
        newstr.replace_range(max_len - 1..str.len(), "…");
        newstr
    } else {
        str
    }
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, batch: &mut DrawBatch) {
    let name_color = ColorPair::new(GRAY80, BLACK);

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
