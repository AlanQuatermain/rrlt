use crate::{prelude::*, KeyState};
use std::path::Path;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

impl Default for MainMenuSelection {
    fn default() -> Self {
        Self::NewGame
    }
}

fn save_exists() -> bool {
    Path::new("./savegame.json").exists()
}

#[system]
#[read_component(Player)]
pub fn main_menu(#[resource] turn_state: &mut TurnState, #[resource] key_state: &KeyState) {
    let selection = if let TurnState::MainMenu { selection } = *turn_state {
        selection
    } else {
        return;
    };

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);

    draw_batch.draw_double_box(
        Rect::with_size(24, 18, 31, 10),
        ColorPair::new(WHEAT, BLACK),
    );
    draw_batch.print_color_centered(20, "Rust Roguelike Tutorial", ColorPair::new(YELLOW, BLACK));
    draw_batch.print_color_centered(21, "by Herbert Wolverson", ColorPair::new(CYAN, BLACK));
    draw_batch.print_color_centered(
        22,
        "Use Up/Down Arrows and Enter",
        ColorPair::new(GRAY, BLACK),
    );

    let selected = ColorPair::new(MAGENTA, BLACK);
    let unselected = ColorPair::new(WHITE, BLACK);

    let mut y_idx = 24;
    if save_exists() {
        draw_batch.print_color_centered(
            y_idx,
            "Continue Game",
            if selection == MainMenuSelection::LoadGame {
                selected
            } else {
                unselected
            },
        );
        y_idx += 1;
    }
    draw_batch.print_color_centered(
        y_idx,
        "Begin New Game",
        if selection == MainMenuSelection::NewGame {
            selected
        } else {
            unselected
        },
    );
    y_idx += 1;
    draw_batch.print_color_centered(
        y_idx,
        "Quit",
        if selection == MainMenuSelection::Quit {
            selected
        } else {
            unselected
        },
    );

    draw_batch.submit(10000).expect("Batch render error");

    if let Some(key) = key_state.key {
        match key {
            VirtualKeyCode::Escape => *turn_state = TurnState::AwaitingInput,
            VirtualKeyCode::Up => {
                let new_selection = match selection {
                    MainMenuSelection::LoadGame => MainMenuSelection::Quit,
                    MainMenuSelection::NewGame => MainMenuSelection::LoadGame,
                    MainMenuSelection::Quit => MainMenuSelection::NewGame,
                };
                *turn_state = TurnState::MainMenu {
                    selection: new_selection,
                }
            }
            VirtualKeyCode::Down => {
                let new_selection = match selection {
                    MainMenuSelection::LoadGame => MainMenuSelection::NewGame,
                    MainMenuSelection::NewGame => MainMenuSelection::Quit,
                    MainMenuSelection::Quit => MainMenuSelection::LoadGame,
                };
                *turn_state = TurnState::MainMenu {
                    selection: new_selection,
                }
            }
            VirtualKeyCode::Return => match selection {
                MainMenuSelection::NewGame => *turn_state = TurnState::NewGame,
                MainMenuSelection::LoadGame => *turn_state = TurnState::LoadGame,
                MainMenuSelection::Quit => ::std::process::exit(0),
            },
            _ => {}
        }
    }
}

#[system]
#[read_component(Player)]
#[write_component(Point)]
#[write_component(Pools)]
#[write_component(Skills)]
#[write_component(Attributes)]
pub fn cheat_menu(
    ecs: &mut SubWorld,
    #[resource] turn_state: &mut TurnState,
    #[resource] key_state: &mut KeyState,
    #[resource] map: &mut Map,
) {
    let mut batch = DrawBatch::new();
    batch.target(2);

    let menu_items = vec![
        ('T', "Teleport to next level"),
        ('H', "Heal to max"),
        ('R', "Reveal the map"),
        ('G', "God mode (no death)"),
        ('L', "Level up"),
    ];

    let y = (25 - (menu_items.len() / 2)) as i32;
    render_menu(
        &mut batch,
        15,
        y,
        31,
        "Cheating!",
        Some("ESCAPE to cancel"),
        &menu_items,
    );

    if let Some(key) = key_state.key {
        match key {
            VirtualKeyCode::T => *turn_state = TurnState::NextLevel,
            VirtualKeyCode::H => {
                <&mut Pools>::query()
                    .filter(component::<Player>())
                    .for_each_mut(ecs, |stats| {
                        stats.hit_points.current = stats.hit_points.max;
                    });
                *turn_state = TurnState::AwaitingInput;
            }
            VirtualKeyCode::R => {
                map.revealed_tiles.iter_mut().for_each(|t| *t = true);
                *turn_state = TurnState::AwaitingInput;
            }
            VirtualKeyCode::G => {
                <&mut Pools>::query()
                    .filter(component::<Player>())
                    .for_each_mut(ecs, |stats| stats.god_mode = true);
                *turn_state = TurnState::AwaitingInput;
            }
            VirtualKeyCode::L => {
                level_up(ecs, map);
                *turn_state = TurnState::AwaitingInput;
            }
            VirtualKeyCode::Escape => *turn_state = TurnState::AwaitingInput,
            _ => {}
        }
    }
}

fn level_up(ecs: &mut SubWorld, map: &Map) {
    <(&Point, &mut Pools, &mut Attributes, &mut Skills)>::query()
        .filter(component::<Player>())
        .for_each_mut(ecs, |(pos, stats, attrs, skills)| {
            // We've gone up a level!
            stats.xp = stats.level * 1000;
            stats.level += 1;
            crate::gamelog::Logger::new()
                .color(MAGENTA)
                .append("Congratulations, you are now level")
                .append(format!("{}", stats.level))
                .log();

            // Improve a random attribute
            let mut rng = RandomNumberGenerator::new();
            match rng.roll_dice(1, 4) {
                1 => {
                    attrs.might.base += 1;
                    crate::gamelog::Logger::new()
                        .color(GREEN)
                        .append("You feel stronger!")
                        .log();
                }
                2 => {
                    attrs.fitness.base += 1;
                    crate::gamelog::Logger::new()
                        .color(GREEN)
                        .append("You feel healthier!")
                        .log();
                }
                3 => {
                    attrs.quickness.base += 1;
                    crate::gamelog::Logger::new()
                        .color(GREEN)
                        .append("You feel quicker!")
                        .log();
                }
                _ => {
                    attrs.intelligence.base += 1;
                    crate::gamelog::Logger::new()
                        .color(GREEN)
                        .append("You feel smarter!")
                        .log();
                }
            }

            // Improve all skills
            for skill in skills.0.iter_mut() {
                *skill.1 += 1;
            }

            stats.hit_points.max =
                player_hp_at_level(attrs.fitness.base + attrs.fitness.modifiers, stats.level);
            stats.hit_points.current = stats.hit_points.max;
            stats.mana.max = mana_at_level(
                attrs.intelligence.base + attrs.intelligence.modifiers,
                stats.level,
            );
            stats.mana.current = stats.mana.max;

            for i in 0..10 {
                if pos.y - i > 1 {
                    add_effect(
                        None,
                        EffectType::Particle {
                            glyph: to_cp437('░'),
                            color: ColorPair::new(GOLD, BLACK),
                            lifespan: 400.0,
                        },
                        Targets::Tile {
                            tile_idx: map.point2d_to_index(*pos - Point::new(0, i)),
                        },
                    );
                }
            }
        });
}
