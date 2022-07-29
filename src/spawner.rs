use crate::prelude::*;

pub enum SpawnType {
    AtPosition { point: Point },
    Equipped { by: Entity },
    Carried { by: Entity },
}

pub fn spawn_player(ecs: &mut World, dm: &MasterDungeonMap, pos: Point) {
    let player = ecs.push((
        Player { map_level: 0 },
        pos,
        Render {
            color: ColorPair::new(YELLOW, BLACK),
            glyph: to_cp437('@'),
            render_order: 1,
        },
        FieldOfView::new(8),
        BlocksTile {},
        SerializeMe,
        HungerClock {
            state: HungerState::WellFed,
            duration: 20,
        },
        Attributes::default(),
        Skills::default(),
        Pools {
            hit_points: Pool {
                current: player_hp_at_level(11, 1),
                max: player_hp_at_level(11, 1),
            },
            mana: Pool {
                current: mana_at_level(11, 1),
                max: mana_at_level(11, 1),
            },
            xp: 0,
            level: 1,
            total_weight: 0.0,
            total_initiative_penalty: 0.0,
            gold: 0.0,
            god_mode: false,
        },
        LightSource {
            color: RGB::from_f32(1.0, 1.0, 0.7),
            range: 8,
        },
        Initiative { current: 0 },
        Faction {
            name: "Player".to_string(),
        },
        EquipmentChanged,
    ));

    let mut commands = CommandBuffer::new(ecs);

    spawn_named_entity(
        &RAWS.lock().unwrap(),
        "Rusty Longsword",
        SpawnType::Equipped { by: player },
        dm,
        &mut commands,
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        "Dried Sausage",
        SpawnType::Carried { by: player },
        dm,
        &mut commands,
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        "Beer",
        SpawnType::Carried { by: player },
        dm,
        &mut commands,
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        "Stained Tunic",
        SpawnType::Equipped { by: player },
        dm,
        &mut commands,
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        "Torn Trousers",
        SpawnType::Equipped { by: player },
        dm,
        &mut commands,
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        "Old Boots",
        SpawnType::Equipped { by: player },
        dm,
        &mut commands,
    );
    spawn_named_entity(
        &RAWS.lock().unwrap(),
        "Town Portal Scroll",
        SpawnType::Carried { by: player },
        dm,
        &mut commands,
    );

    let mut resources = Resources::default(); // unused in this instance
    commands.flush(ecs, &mut resources);
}

pub fn spawn_town_portal(ecs: &mut World, resources: &mut Resources) {
    // Get current position & depth
    let map = resources.get::<Map>().unwrap();
    let depth = map.depth;
    let player_pos = <&Point>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap()
        .clone();
    std::mem::drop(map);

    // Find part of the town for the portal
    let dm = resources.get::<MasterDungeonMap>().unwrap();
    let town_map = dm.get_map(0).unwrap();
    let mut stairs_idx = 0;
    for (idx, tt) in town_map.tiles.iter().enumerate() {
        if *tt == TileType::DownStairs {
            stairs_idx = idx;
            break;
        }
    }
    let portal_pos = town_map.index_to_point2d(stairs_idx);
    std::mem::drop(dm);

    // Spawn the portal itself
    ecs.push((
        OtherLevelPosition {
            position: portal_pos,
            depth: 0,
        },
        Render {
            color: ColorPair::new(CYAN, BLACK),
            glyph: to_cp437('♥'),
            render_order: 0,
        },
        EntryTrigger,
        TeleportTo {
            position: player_pos,
            depth,
            player_only: true,
        },
        Name("Town Portal".to_string()),
        SingleActivation,
    ));
}

#[allow(dead_code)]
pub fn spawn_amulet_of_yala(ecs: &mut World, pos: Point) {
    ecs.push((
        Item {
            initiative_penalty: 0.0,
            weight_lbs: 0.1,
            base_value: 10000.0,
        },
        AmuletOfYala,
        pos,
        Render {
            color: ColorPair::new(BLUEVIOLET, BLACK),
            glyph: to_cp437('|'),
            render_order: 1,
        },
        Name("Amulet of Yala".to_string()),
    ));
}

pub fn spawn_entity(ecs: &mut World, dm: &MasterDungeonMap, spawn: &(&Point, &String)) {
    let pos = *spawn.0;

    let mut command_buffer = CommandBuffer::new(ecs);
    if spawn_named_entity(
        &RAWS.lock().unwrap(),
        &spawn.1,
        SpawnType::AtPosition { point: pos },
        dm,
        &mut command_buffer,
    )
    .is_some()
    {
        // dummy resources, they won't be needed
        let mut resources = Resources::default();
        command_buffer.flush(ecs, &mut resources);
        return;
    }

    log(format!(
        "WARNING: We don't know how to spawn [{}]!",
        spawn.1
    ));
}
