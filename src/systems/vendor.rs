use crate::{prelude::*, KeyState};

#[system]
#[read_component(Item)]
#[read_component(Vendor)]
#[read_component(Carried)]
#[write_component(Pools)]
#[read_component(Name)]
#[read_component(ObfuscatedName)]
pub fn vendor(
    ecs: &mut SubWorld,
    #[resource] turn_state: &mut TurnState,
    #[resource] key_state: &mut KeyState,
    #[resource] dm: &MasterDungeonMap,
    commands: &mut CommandBuffer,
) {
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap()
        .clone();
    match *turn_state {
        TurnState::ShowingVendor { vendor, mode } => {
            let new_state = match mode {
                VendorMode::Buy => {
                    vendor_buy_menu(ecs, vendor, player_entity, key_state, dm, commands)
                }
                VendorMode::Sell => {
                    vendor_sell_menu(ecs, vendor, player_entity, key_state, commands)
                }
            };
            if let Some(updated) = new_state {
                *turn_state = updated;
            }
        }
        _ => {}
    }
}

fn vendor_sell_menu(
    ecs: &mut SubWorld,
    vendor: Entity,
    player: Entity,
    key_state: &mut KeyState,
    commands: &mut CommandBuffer,
) -> Option<TurnState> {
    let player_entity = <Entity>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();

    let inventory: Vec<(Name, Option<ObfuscatedName>, Item, Entity)> =
        <(&Carried, &Name, Option<&ObfuscatedName>, &Item, Entity)>::query()
            .iter(ecs)
            .filter(|(c, _, _, _, _)| c.0 == *player_entity)
            .map(|(_, n, o, i, e)| (n.clone(), o.map(|v| v.clone()), i.clone(), *e))
            .collect();
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;
    let mut batch = DrawBatch::new();
    batch.target(2);

    batch.draw_box(
        Rect::with_size(15, y - 2, 51, (count + 3) as i32),
        ColorPair::new(WHITE, BLACK),
    );
    batch.print_color(
        Point::new(18, y - 2),
        "Sell Which Item? (space to switch to buy mode)",
        ColorPair::new(YELLOW, BLACK),
    );
    batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(YELLOW, BLACK),
    );

    let mut equippable: Vec<Entity> = Vec::new();
    let mut j = 0;
    for (name, oname, item, entity) in inventory {
        batch.set(
            Point::new(17, y),
            ColorPair::new(WHITE, BLACK),
            to_cp437('('),
        );
        batch.set(
            Point::new(18, y),
            ColorPair::new(YELLOW, BLACK),
            97 + j as FontCharType,
        );
        batch.set(
            Point::new(19, y),
            ColorPair::new(WHITE, BLACK),
            to_cp437(')'),
        );

        if let Some(obfuscated) = oname {
            batch.print(Point::new(21, y), &obfuscated.0.clone());
        } else {
            batch.print(Point::new(21, y), &name.0.clone());
        }
        batch.print(
            Point::new(50, y),
            &format!("{:.1} gp", item.base_value * 0.8),
        );
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    batch.submit(12000).expect("Batch error");

    if let Some(key) = key_state.key {
        return match key {
            VirtualKeyCode::Space => Some(TurnState::ShowingVendor {
                vendor,
                mode: VendorMode::Buy,
            }),
            VirtualKeyCode::Escape => Some(TurnState::AwaitingInput),
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    sell_item(equippable[selection as usize], player, ecs, commands);
                }
                None
            }
        };
    }

    None
}

fn vendor_buy_menu(
    ecs: &mut SubWorld,
    vendor: Entity,
    player: Entity,
    key_state: &mut KeyState,
    dm: &MasterDungeonMap,
    commands: &mut CommandBuffer,
) -> Option<TurnState> {
    let vendor_entry = ecs.entry_ref(vendor).unwrap();
    let categories = vendor_entry
        .get_component::<Vendor>()
        .unwrap()
        .categories
        .clone();
    std::mem::drop(vendor_entry);

    let raws = &RAWS.lock().unwrap();

    let inventory = get_vendor_items(&categories, raws);
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;
    let mut batch = DrawBatch::new();
    batch.target(2);

    batch.draw_box(
        Rect::with_size(15, y - 2, 51, (count + 3) as i32),
        ColorPair::new(WHITE, BLACK),
    );
    batch.print_color(
        Point::new(18, y - 2),
        "Buy Which Item? (space to switch to sell mode)",
        ColorPair::new(YELLOW, BLACK),
    );
    batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(YELLOW, BLACK),
    );

    for (j, sale) in inventory.iter().enumerate() {
        batch.set(
            Point::new(17, y),
            ColorPair::new(WHITE, BLACK),
            to_cp437('('),
        );
        batch.set(
            Point::new(18, y),
            ColorPair::new(YELLOW, BLACK),
            97 + j as FontCharType,
        );
        batch.set(
            Point::new(19, y),
            ColorPair::new(WHITE, BLACK),
            to_cp437(')'),
        );

        batch.print(Point::new(21, y), &sale.0.clone());
        batch.print(Point::new(50, y), &format!("{:.1} gp", sale.1 * 1.2));
        y += 1;
    }

    batch.submit(12000).expect("Batch error");

    if let Some(key) = key_state.key {
        return match key {
            VirtualKeyCode::Space => Some(TurnState::ShowingVendor {
                vendor,
                mode: VendorMode::Sell,
            }),
            VirtualKeyCode::Escape => Some(TurnState::AwaitingInput),
            _ => {
                let selection = letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    buy_item(
                        inventory[selection as usize].0.clone(),
                        inventory[selection as usize].1,
                        player,
                        ecs,
                        commands,
                        dm,
                        raws,
                    );
                }
                None
            }
        };
    }

    None
}

fn sell_item(entity: Entity, player: Entity, ecs: &mut SubWorld, commands: &mut CommandBuffer) {
    let entry = ecs.entry_ref(entity).unwrap();
    let item = entry.get_component::<Item>().unwrap();
    let price = item.base_value * 0.8;
    let weight = item.weight_lbs;
    std::mem::drop(item);
    std::mem::drop(entry);

    if let Ok(mut stats) = ecs.entry_mut(player).unwrap().get_component_mut::<Pools>() {
        stats.gold += price;
        stats.total_weight -= weight; // applying weight change directly, no EquipmentChanged
    }
    commands.remove(entity);
}

fn buy_item(
    name: String,
    price: f32,
    player: Entity,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    dm: &MasterDungeonMap,
    raws: &RawMaster,
) {
    if let Ok(stats) = ecs.entry_mut(player).unwrap().get_component_mut::<Pools>() {
        if stats.gold >= price {
            stats.gold -= price;
            if let Some(entity) =
                spawn_named_item(raws, &name, SpawnType::Carried { by: player }, dm, commands)
            {
                commands.add_component(entity, IdentifiedItem(name.clone()));
            }

            commands.add_component(player, EquipmentChanged); // trigger encumbrance update
        }
    }
}
