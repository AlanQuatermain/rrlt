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
                VendorMode::Buy { page } => {
                    vendor_buy_menu(ecs, vendor, page, player_entity, key_state, dm, commands)
                }
                VendorMode::Sell { page } => {
                    vendor_sell_menu(ecs, vendor, page, player_entity, key_state, commands)
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
    page: i32,
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

    // Reduce to an array of (entity, title, cost)
    let items: Vec<_> = inventory
        .iter()
        .map(|(name, oname, item, _)| {
            if let Some(oname) = oname {
                (oname.0.clone(), item.base_value * 0.8)
            } else {
                (name.0.clone(), item.base_value * 0.8)
            }
        })
        .collect();

    let mut batch = DrawBatch::new();
    batch.target(2);

    let result = vendor_result_menu(
        &mut batch,
        "Sell Which Item? (space to switch to buy mode)",
        page,
        &items,
        key_state.key,
    );

    match result.0 {
        VendorMenuResult::NoResponse => None,
        VendorMenuResult::Cancel => Some(TurnState::AwaitingInput),
        VendorMenuResult::BuySellToggle => Some(TurnState::ShowingVendor {
            vendor,
            mode: VendorMode::Buy { page: 0 },
        }),
        VendorMenuResult::PreviousPage => Some(TurnState::ShowingVendor {
            vendor,
            mode: VendorMode::Sell { page: page - 1 },
        }),
        VendorMenuResult::NextPage => Some(TurnState::ShowingVendor {
            vendor,
            mode: VendorMode::Sell { page: page + 1 },
        }),
        VendorMenuResult::Selected => {
            sell_item(inventory[result.1.unwrap()].3, player, ecs, commands);
            None
        }
    }
}

fn vendor_buy_menu(
    ecs: &mut SubWorld,
    vendor: Entity,
    page: i32,
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
    let mut batch = DrawBatch::new();
    batch.target(2);

    let result = vendor_result_menu(
        &mut batch,
        "Buy Which Item? (space to switch to sell mode)",
        page,
        &inventory,
        key_state.key,
    );

    match result.0 {
        VendorMenuResult::NoResponse => None,
        VendorMenuResult::Cancel => Some(TurnState::AwaitingInput),
        VendorMenuResult::BuySellToggle => Some(TurnState::ShowingVendor {
            vendor,
            mode: VendorMode::Sell { page: 0 },
        }),
        VendorMenuResult::PreviousPage => Some(TurnState::ShowingVendor {
            vendor,
            mode: VendorMode::Buy { page: page - 1 },
        }),
        VendorMenuResult::NextPage => Some(TurnState::ShowingVendor {
            vendor,
            mode: VendorMode::Buy { page: page + 1 },
        }),
        VendorMenuResult::Selected => {
            let item = &inventory[result.1.unwrap()];
            buy_item(item.0.clone(), item.1, player, ecs, commands, dm, raws);
            None
        }
    }
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
