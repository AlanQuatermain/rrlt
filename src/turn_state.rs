use crate::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VendorMode {
    Buy,
    Sell,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    AwaitingInput,
    Ticking,
    RevealMap { row: i32 },

    ShowingInventory,
    ShowingDropItems,
    ShowingVendor { vendor: Entity, mode: VendorMode },
    ShowingRemoveCurse,
    ShowingIdentify,

    RangedTargeting { range: i32, item: Entity },

    MainMenu { selection: MainMenuSelection },

    NewGame,
    SaveGame,
    LoadGame,
    GameOver,

    NextLevel,
    PreviousLevel,
    TownPortal,
    LevelTeleport { destination: Point, depth: i32 },

    MapBuilding { step: usize },
    ShowCheatMenu,
}
