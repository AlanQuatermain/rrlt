use crate::prelude::*;

pub enum SpawnType {
    AtPosition { point: Point },
}

pub fn spawn_player(ecs: &mut World, pos: Point) {
    ecs.push((
        Player { map_level: 0 },
        pos,
        Render {
            color: ColorPair::new(YELLOW, BLACK),
            glyph: to_cp437('@'),
            render_order: 1,
        },
        FieldOfView::new(8),
        Damage(5),
        Armor(2),
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
        },
    ));
}

#[allow(dead_code)]
pub fn spawn_amulet_of_yala(ecs: &mut World, pos: Point) {
    ecs.push((
        Item,
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

pub fn spawn_entity(ecs: &mut World, spawn: &(&Point, &String)) {
    let pos = *spawn.0;

    let mut command_buffer = CommandBuffer::new(ecs);
    if spawn_named_entity(
        &RAWS.lock().unwrap(),
        &spawn.1,
        SpawnType::AtPosition { point: pos },
        &mut command_buffer,
    ) {
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
