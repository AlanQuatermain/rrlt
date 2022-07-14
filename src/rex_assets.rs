use crate::prelude::*;

embedded_resource!(SMALL_DUNGEON, "../resources/SmallDungeon_80x50.xp");
embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
// embedded_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

pub struct RexAssets {
    pub menu: XpFile
}

impl RexAssets {
    pub fn new() -> RexAssets {
        link_resource!(SMALL_DUNGEON, "../resources/SmallDungeon_80x50.xp");
        link_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
        // link_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");
        Self {
            menu: XpFile::from_resource("../resources/SmallDungeon_80x50.xp").unwrap()
        }
    }
}