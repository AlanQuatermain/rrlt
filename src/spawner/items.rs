use crate::prelude::*;

pub fn spawn_amulet_of_yala(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, AmuletOfYala, pos,
            Render { color: ColorPair::new(BLUEVIOLET, BLACK), glyph: to_cp437('|') },
            Name("Amulet of Yala".to_string())
        )
    );
}

pub fn spawn_random_item(ecs: &mut World, rng: &mut RandomNumberGenerator, pos: Point) {
    match rng.roll_dice(1, 4) {
        1 => health_potion(ecs, 8, pos),
        2 => magic_missile_scroll(ecs, pos),
        3 => fireball_scroll(ecs, pos),
        _ => confusion_scroll(ecs, pos),
    }
}

pub fn health_potion(ecs: &mut World, healing: i32, pos: Point) {
    ecs.push(
        (
            Item, Consumable{}, pos.clone(),
            Render { color: ColorPair::new(MAGENTA, BLACK), glyph: to_cp437(';') },
            ProvidesHealing { amount: healing }, Name("Healing Potion".to_string()),
        )
    );
}

pub fn magic_missile_scroll(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, Consumable{}, pos.clone(),
            Render { color: ColorPair::new(CYAN, BLACK), glyph: to_cp437(')') },
            Name("Magic Missile Scroll".to_string()), Damage(8), Ranged(6), SerializeMe
        )
    );
}

pub fn fireball_scroll(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, Consumable, pos.clone(),
            Render { color: ColorPair::new(ORANGE, BLACK), glyph: to_cp437(')') },
            Name("Fireball Scroll".to_string()), Damage(20), Ranged(6), AreaOfEffect(3),
            SerializeMe
        )
    );
}

pub fn confusion_scroll(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, Consumable, pos.clone(),
            Render { color: ColorPair::new(PINK, BLACK), glyph: to_cp437(')') },
            Name("Confusion Scroll".to_string()), Ranged(6), Confusion(4), SerializeMe
        )
    );
}

pub fn dagger(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, Weapon, pos.clone(),
            Render { color: ColorPair::new(CYAN, BLACK), glyph: to_cp437('/') },
            Name("Dagger".to_string()), Damage(2), SerializeMe,
            Equippable { slot: EquipmentSlot::Melee }
        )
    );
}

pub fn longsword(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, Weapon, pos.clone(),
            Render { color: ColorPair::new(YELLOW, BLACK), glyph: to_cp437('/') },
            Name("Longsword".to_string()), Damage(4), SerializeMe,
            Equippable { slot: EquipmentSlot::Melee }
        )
    );
}

pub fn shield(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, Weapon, pos.clone(),
            Render { color: ColorPair::new(CYAN, BLACK), glyph: to_cp437('(') },
            Name("Shield".to_string()), Armor(1), SerializeMe,
            Equippable { slot: EquipmentSlot::Shield }
        )
    );
}

pub fn tower_shield(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, Weapon, pos.clone(),
            Render { color: ColorPair::new(YELLOW, BLACK), glyph: to_cp437('(') },
            Name("Tower Shield".to_string()), Armor(3), SerializeMe,
            Equippable { slot: EquipmentSlot::Shield }
        )
    );
}

pub fn rations(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, ProvidesFood, pos.clone(),
            Render { color: ColorPair::new(GREEN, BLACK), glyph: to_cp437('%') },
            Name("Rations".to_string()), SerializeMe, Consumable,
        )
    );
}

pub fn dungeon_map(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Item, ProvidesDungeonMap, pos.clone(),
            Render { color: ColorPair::new(CYAN3, BLACK), glyph: to_cp437(')') },
            Name("Dungeon Map".to_string()), SerializeMe, Consumable
        )
    );
}

pub fn bear_trap(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            pos.clone(), Hidden, EntryTrigger, Damage(6), SingleActivation,
            Render { color: ColorPair::new(RED, BLACK), glyph: to_cp437('^') },
            Name("Bear Trap".to_string()), SerializeMe
        )
    );
}