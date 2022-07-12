mod monsters;
mod items;

use crate::prelude::*;
pub use monsters::*;
pub use items::*;

pub fn spawn_player(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Player{ map_level: 0 },
            pos,
            Render {
                color: ColorPair::new(YELLOW, BLACK),
                glyph: to_cp437('@')
            },
            Health { current: 30, max: 30 },
            FieldOfView::new(8),
            Damage(5),
            Armor(2),
            BlocksTile{},
            SerializeMe,
            HungerClock { state: HungerState::WellFed, duration: 20 },
        )
    );
}

pub fn spawn_mob(
    ecs: &mut World,
    pos: Point,
    spawn_table: &RandomTable,
    rng: &mut RandomNumberGenerator
) {
    match spawn_table.roll(rng).as_str() {
        "Goblin" => goblin(ecs, pos),
        "Orc" => orc(ecs, pos),
        "Ogre" => ogre(ecs, pos),
        "Ettin" => ettin(ecs, pos),
        "Health Potion" => health_potion(ecs, 8, pos),
        "Fireball Scroll" => fireball_scroll(ecs, pos),
        "Confusion Scroll" => confusion_scroll(ecs, pos),
        "Magic Missile Scroll" => magic_missile_scroll(ecs, pos),
        "Dagger" => dagger(ecs, pos),
        "Shield" => shield(ecs, pos),
        "Longsword" => longsword(ecs, pos),
        "Tower Shield" => tower_shield(ecs, pos),
        "Rations" => rations(ecs, pos),
        "Dungeon Map" => dungeon_map(ecs, pos),
        "Bear Trap" => bear_trap(ecs, pos),
        _ => {}
    }
}