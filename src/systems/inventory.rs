use crate::{prelude::*, KeyState};

#[system]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Wearable)]
#[read_component(Damage)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(Ranged)]
#[read_component(Equippable)]
#[read_component(Equipped)]
#[read_component(MagicItem)]
#[read_component(ObfuscatedName)]
pub fn inventory(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] key_state: &mut KeyState,
    #[resource] turn_state: &mut TurnState,
    #[resource] dm: &MasterDungeonMap,
) {
    match *turn_state {
        TurnState::ShowingInventory | TurnState::ShowingDropItems => {}
        _ => return,
    }

    let player = <(Entity, &Player)>::query()
        .iter(ecs)
        .find_map(|(entity, _)| Some(*entity))
        .unwrap();

    let count = <(&Item, &Carried)>::query()
        .iter(ecs)
        .filter(|(_, carried)| carried.0 == player)
        .count();
    let mut item_query = <(&Item, &Name, &Carried, Entity)>::query();

    // build item/name list
    let items: Vec<(Entity, String)> = item_query
        .iter(ecs)
        .filter(|(_, _, carried, _)| carried.0 == player)
        .map(|(_, _, _, entity)| (*entity, get_item_display_name(ecs, *entity, dm)))
        .collect();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    // determine required width
    // 2 for each border+margin, 4 for key+space, 3 for space+E+space
    // 20 was the original width
    let width = usize::max(
        items.iter().map(|i| i.1.len()).max().unwrap_or(20) + 2 + 2 + 4 + 3,
        20,
    ) as i32;

    let title = match *turn_state {
        TurnState::ShowingDropItems => "Drop Item".to_string(),
        _ => "Inventory".to_string(),
    };

    let mut y = (25 - (count / 2)) as i32;
    draw_batch.draw_box(
        Rect::with_size(15, y - 2, width, (count + 3) as i32),
        ColorPair::new(WHITE, BLACK),
    );
    draw_batch.print_color(Point::new(18, y - 2), title, ColorPair::new(YELLOW, BLACK));
    draw_batch.print_color(
        Point::new(18, y + (count as i32) + 1),
        "ESCAPE to cancel",
        ColorPair::new(YELLOW, BLACK),
    );

    let mut usable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, name) in items.iter() {
        draw_batch.set(
            Point::new(17, y),
            ColorPair::new(WHITE, BLACK),
            to_cp437('('),
        );
        draw_batch.set(
            Point::new(18, y),
            ColorPair::new(YELLOW, BLACK),
            97 + j as FontCharType,
        );
        draw_batch.set(
            Point::new(19, y),
            ColorPair::new(WHITE, BLACK),
            to_cp437(')'),
        );

        draw_batch.print_color(Point::new(21, y), &name, get_item_color(ecs, *entity));
        if let Ok(entry) = ecs.entry_ref(*entity) {
            if entry.get_component::<Equipped>().is_ok() {
                draw_batch.set(
                    Point::new(15 + width - 3, y),
                    ColorPair::new(YELLOW, BLACK),
                    to_cp437('E'),
                );
            } else if entry.get_component::<Equippable>().is_ok() {
                draw_batch.set(
                    Point::new(15 + width - 3, y),
                    ColorPair::new(WHITE, BLACK),
                    to_cp437('E'),
                );
            }
        }
        usable.push(*entity);
        y += 1;
        j += 1;
    }

    draw_batch.submit(12000).expect("Batch error");

    if let Some(input) = key_state.key {
        match input {
            VirtualKeyCode::Escape => *turn_state = TurnState::AwaitingInput,
            _ => {
                let selection = letter_to_option(input);
                if selection > -1 && selection < count as i32 {
                    match *turn_state {
                        TurnState::ShowingInventory => {
                            let item = usable[selection as usize];
                            let entry = ecs.entry_ref(item).unwrap();
                            if let Ok(range) = entry.get_component::<Ranged>() {
                                *turn_state = TurnState::RangedTargeting {
                                    range: range.0,
                                    item,
                                };
                                return;
                            } else {
                                commands.add_component(
                                    usable[selection as usize],
                                    UseItem {
                                        user: player,
                                        target: None,
                                    },
                                );
                            }
                        }
                        TurnState::ShowingDropItems => {
                            commands.push((
                                (),
                                WantsToDrop {
                                    who: player,
                                    what: usable[selection as usize],
                                },
                            ));
                        }
                        _ => return,
                    }

                    *turn_state = TurnState::Ticking;
                }
            }
        }
        key_state.key = None;
    }
}

#[system(for_each)]
#[read_component(IdentifiedItem)]
#[read_component(MagicItem)]
#[read_component(Player)]
#[read_component(Name)]
pub fn identification(
    entity: &Entity,
    id_info: &IdentifiedItem,
    carried: &Carried,
    name: &Name,
    #[resource] dm: &mut MasterDungeonMap,
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
) {
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();
    if carried.0 != *player_entity {
        return;
    }

    if !dm.identified_items.contains(&name.0) && is_tag_magic(&name.0) {
        dm.identified_items.insert(name.0.clone());
    }

    <(Entity, &Name)>::query()
        .filter(component::<ObfuscatedName>())
        .iter(ecs)
        .filter(|(_, n)| n.0 == name.0)
        .for_each(|(e, _)| commands.remove_component::<ObfuscatedName>(*e));

    commands.remove_component::<IdentifiedItem>(*entity);
}
