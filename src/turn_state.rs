use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    AwaitingInput,
    Ticking,
    RevealMap { row: i32 },

    ShowingInventory,
    ShowingDropItems,

    RangedTargeting { range: i32, item: Entity },

    MainMenu { selection: MainMenuSelection },

    NewGame,
    SaveGame,
    LoadGame,
    GameOver,

    NextLevel,
    PreviousLevel,

    MapBuilding { step: usize },
    ShowCheatMenu,
}
