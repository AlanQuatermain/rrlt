use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,

    ShowingInventory,
    ShowingDropItems,

    RangedTargeting { range: i32, item: Entity },

    MainMenu { selection: MainMenuSelection },

    NewGame,
    SaveGame,
    LoadGame,
    GameOver,

    NextLevel,
}