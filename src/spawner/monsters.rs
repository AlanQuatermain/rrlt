use crate::prelude::*;

#[allow(dead_code)]
pub fn spawn_monster(ecs: &mut World, rng: &mut RandomNumberGenerator, pos: Point) {
    match rng.roll_dice(1, 16) {
        1..=12 => goblin(ecs, pos),
        13..=17 => orc(ecs, pos),
        18..=19 => ogre(ecs, pos),
        _ => ettin(ecs, pos)
    };
}

pub fn goblin(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Enemy,
            pos,
            Render {
                color: ColorPair::new(RED, BLACK),
                glyph: to_cp437('g')
            },
            FieldOfView::new(6),
            Name("Goblin".to_string()),
            SerializeMe,
            ChasingPlayer{},
            Health{ current: 5, max: 5 },
            Damage(3),
            Armor(0)
        )
    );
}

pub fn orc(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Enemy,
            pos,
            Render {
                color: ColorPair::new(RED, BLACK),
                glyph: to_cp437('o')
            },
            FieldOfView::new(6),
            Name("Orc".to_string()),
            SerializeMe,
            ChasingPlayer{},
            Health{ current: 8, max: 8 },
            Damage(3),
            Armor(2)
        )
    );
}

pub fn ogre(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Enemy,
            pos,
            Render {
                color: ColorPair::new(RED, BLACK),
                glyph: to_cp437('O')
            },
            FieldOfView::new(6),
            Name("Ogre".to_string()),
            SerializeMe,
            ChasingPlayer{},
            Health{ current: 12, max: 12 },
            Damage(4),
            Armor(2)
        )
    );
}

pub fn ettin(ecs: &mut World, pos: Point) {
    ecs.push(
        (
            Enemy,
            pos,
            Render {
                color: ColorPair::new(RED, BLACK),
                glyph: to_cp437('E')
            },
            FieldOfView::new(6),
            Name("Ettin".to_string()),
            SerializeMe,
            ChasingPlayer{},
            Health{ current: 16, max: 16 },
            Damage(4),
            Armor(1)
        )
    );
}