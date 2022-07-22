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
pub fn cheat_menu(
    ecs: &mut SubWorld,
    #[resource] turn_state: &mut TurnState,
    #[resource] key_state: &mut KeyState,
) {
    let mut batch = DrawBatch::new();
    batch.target(2);

    let yellow = ColorPair::new(YELLOW, BLACK);
    let white = ColorPair::new(WHITE, BLACK);

    let count = 3;
    let mut y = (25 - (count / 2)) as i32;
    batch.draw_box(Rect::with_size(15, y - 2, 31, count + 3), white);
    batch.print_color(Point::new(18, y - 2), "Cheating!", yellow);
    batch.print_color(Point::new(18, y + count), "ESCAPE to cancel", yellow);

    batch.set(Point::new(17, y), white, to_cp437('('));
    batch.set(Point::new(18, y), yellow, to_cp437('T'));
    batch.set(Point::new(19, y), white, to_cp437(')'));
    batch.print(Point::new(21, y), "Teleport to next level");

    y += 1;

    batch.set(Point::new(17, y), white, to_cp437('('));
    batch.set(Point::new(18, y), yellow, to_cp437('H'));
    batch.set(Point::new(19, y), white, to_cp437(')'));
    batch.print(Point::new(21, y), "Heal to max");

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
            VirtualKeyCode::Escape => *turn_state = TurnState::AwaitingInput,
            _ => {}
        }
    }
}
