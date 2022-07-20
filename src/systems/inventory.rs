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
pub fn inventory(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] key_state: &KeyState,
    #[resource] turn_state: &mut TurnState,
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

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    let title = match *turn_state {
        TurnState::ShowingDropItems => "Drop Item".to_string(),
        _ => "Inventory".to_string(),
    };

    let mut y = (25 - (count / 2)) as i32;
    draw_batch.draw_box(
        Rect::with_size(15, y - 2, 31, (count + 3) as i32),
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
    for (name, entity) in item_query
        .iter(ecs)
        .filter(|(_, _, carried, _)| carried.0 == player)
        .map(|(_, name, _, entity)| (&name.0, entity))
    {
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

        draw_batch.print(Point::new(21, y), &name);
        if let Ok(entry) = ecs.entry_ref(*entity) {
            if entry.get_component::<Equipped>().is_ok() {
                draw_batch.set(
                    Point::new(44, y),
                    ColorPair::new(YELLOW, BLACK),
                    to_cp437('E'),
                );
            } else if entry.get_component::<Equippable>().is_ok() {
                draw_batch.set(
                    Point::new(44, y),
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
                                commands.push((
                                    (),
                                    ActivateItem {
                                        used_by: player,
                                        item: usable[selection as usize],
                                        target: None,
                                    },
                                ));
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

                    *turn_state = TurnState::PlayerTurn;
                }
            }
        }
    }
}
