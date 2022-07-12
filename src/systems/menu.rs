use std::path::Path;
use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit
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
pub fn main_menu(
    ecs: &SubWorld,
    #[resource] turn_state: &mut TurnState,
    #[resource] key: &mut Option<VirtualKeyCode>,
    commands: &mut CommandBuffer
) {
    let selection = if let TurnState::MainMenu {selection} = *turn_state {
        selection
    }
    else {
        return
    };

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);

    let selected = ColorPair::new(MAGENTA, BLACK);
    let unselected = ColorPair::new(WHITE, BLACK);

    let mut y_idx = 24;
    if save_exists() {
        draw_batch.print_color_centered(y_idx, "Continue Game",
                                        if selection == MainMenuSelection::LoadGame { selected } else { unselected });
        y_idx += 1;
    }
    draw_batch.print_color_centered(y_idx, "Begin New Game",
                                    if selection == MainMenuSelection::NewGame { selected } else { unselected });
    y_idx += 1;
    draw_batch.print_color_centered(y_idx, "Quit",
                                    if selection == MainMenuSelection::Quit { selected } else { unselected });

    draw_batch.submit(10000)
        .expect("Batch render error");

    if let Some(key) = key {
        match key {
            VirtualKeyCode::Escape => *turn_state = TurnState::AwaitingInput,
            VirtualKeyCode::Up => {
                let new_selection = match selection {
                    MainMenuSelection::LoadGame => MainMenuSelection::Quit,
                    MainMenuSelection::NewGame => MainMenuSelection::LoadGame,
                    MainMenuSelection::Quit => MainMenuSelection::NewGame,
                };
                *turn_state = TurnState::MainMenu {selection: new_selection}
            },
            VirtualKeyCode::Down => {
                let new_selection = match selection {
                    MainMenuSelection::LoadGame => MainMenuSelection::NewGame,
                    MainMenuSelection::NewGame => MainMenuSelection::Quit,
                    MainMenuSelection::Quit => MainMenuSelection::LoadGame,
                };
                *turn_state = TurnState::MainMenu {selection: new_selection}
            },
            VirtualKeyCode::Return => {
                match selection {
                    MainMenuSelection::NewGame => { *turn_state = TurnState::NewGame },
                    MainMenuSelection::LoadGame => { *turn_state = TurnState::LoadGame },
                    MainMenuSelection::Quit => { ::std::process::exit(0) }
                }
            },
            _ => {}
        }
    }
}