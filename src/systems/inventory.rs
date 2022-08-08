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
#[read_component(CursedItem)]
pub fn inventory(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] key_state: &mut KeyState,
    #[resource] turn_state: &mut TurnState,
    #[resource] dm: &MasterDungeonMap,
) {
    match *turn_state {
        TurnState::ShowingInventory
        | TurnState::ShowingDropItems
        | TurnState::ShowingRemoveCurse
        | TurnState::ShowingIdentify => {}
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
    let mut item_query = <(
        &Carried,
        Option<&CursedItem>,
        Option<&ObfuscatedName>,
        Entity,
    )>::query()
    .filter(component::<Item>());

    // build item/name list
    let items: Vec<(Entity, String)> = item_query
        .iter(ecs)
        .filter(|(carried, _, _, _)| carried.0 == player)
        .filter(|(_, cursed, obfname, _)| match *turn_state {
            TurnState::ShowingRemoveCurse => cursed.is_some(),
            TurnState::ShowingIdentify => obfname.is_some(),
            _ => true,
        })
        .map(|(_, _, _, entity)| (*entity, get_item_display_name(ecs, *entity, dm)))
        .collect();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);

    // // determine required width
    // // 2 for each border+margin, 4 for key+space, 3 for space+E+space
    // // 20 was the original width
    // let width = usize::max(
    //     items.iter().map(|i| i.1.len()).max().unwrap_or(20) + 2 + 2 + 4 + 3,
    //     20,
    // ) as i32;

    let title = match *turn_state {
        TurnState::ShowingDropItems => "Drop Item".to_string(),
        TurnState::ShowingRemoveCurse => "Remove Curse".to_string(),
        TurnState::ShowingIdentify => "Identify".to_string(),
        _ => "Inventory".to_string(),
    };

    // let mut y = (25 - (count / 2)) as i32;
    let result = item_result_menu(&mut draw_batch, title, count, &items, key_state.key);

    // draw_batch.draw_box(
    //     Rect::with_size(15, y - 2, width, (count + 3) as i32),
    //     ColorPair::new(WHITE, BLACK),
    // );
    // draw_batch.print_color(Point::new(18, y - 2), title, ColorPair::new(YELLOW, BLACK));
    // draw_batch.print_color(
    //     Point::new(18, y + (count as i32) + 1),
    //     "ESCAPE to cancel",
    //     ColorPair::new(YELLOW, BLACK),
    // );

    // let mut usable: Vec<Entity> = Vec::new();
    // let mut j = 0;
    // for (entity, name) in items.iter() {
    //     draw_batch.set(
    //         Point::new(17, y),
    //         ColorPair::new(WHITE, BLACK),
    //         to_cp437('('),
    //     );
    //     draw_batch.set(
    //         Point::new(18, y),
    //         ColorPair::new(YELLOW, BLACK),
    //         97 + j as FontCharType,
    //     );
    //     draw_batch.set(
    //         Point::new(19, y),
    //         ColorPair::new(WHITE, BLACK),
    //         to_cp437(')'),
    //     );

    //     draw_batch.print_color(Point::new(21, y), &name, get_item_color(ecs, *entity, dm));
    //     if let Ok(entry) = ecs.entry_ref(*entity) {
    //         if entry.get_component::<Equipped>().is_ok() {
    //             draw_batch.set(
    //                 Point::new(15 + width - 3, y),
    //                 ColorPair::new(YELLOW, BLACK),
    //                 to_cp437('E'),
    //             );
    //         } else if entry.get_component::<Equippable>().is_ok() {
    //             draw_batch.set(
    //                 Point::new(15 + width - 3, y),
    //                 ColorPair::new(WHITE, BLACK),
    //                 to_cp437('E'),
    //             );
    //         }
    //     }
    //     usable.push(*entity);
    //     y += 1;
    //     j += 1;
    // }

    match result.0 {
        ItemMenuResult::Cancel => *turn_state = TurnState::AwaitingInput,
        ItemMenuResult::Selected => {
            let item = result.1.unwrap();
            match *turn_state {
                TurnState::ShowingInventory => {
                    let entry = ecs.entry_ref(item).unwrap();
                    if let Ok(range) = entry.get_component::<Ranged>() {
                        *turn_state = TurnState::RangedTargeting {
                            range: range.0,
                            item,
                        };
                        return;
                    } else {
                        commands.add_component(
                            item,
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
                            what: item,
                        },
                    ));
                }
                TurnState::ShowingRemoveCurse => {
                    commands.remove_component::<CursedItem>(item);
                }
                TurnState::ShowingIdentify => {
                    add_effect(
                        Some(player),
                        EffectType::Identify,
                        Targets::Single { target: item },
                    );
                }
                _ => return,
            }

            *turn_state = TurnState::Ticking;
        }
        _ => {}
    }
    key_state.key = None;
}

#[system(for_each)]
#[read_component(IdentifiedItem)]
#[read_component(MagicItem)]
#[read_component(Player)]
#[read_component(Name)]
pub fn identification(
    entity: &Entity,
    _id_info: &IdentifiedItem,
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
