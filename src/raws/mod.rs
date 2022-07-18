use crate::prelude::*;
use std::sync::Mutex;

mod item_structs;
mod mob_structs;
mod prop_structs;
mod rawmaster;

pub use rawmaster::*;

lazy_static! {
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::empty());
}

embedded_resource!(RAW_FILE, "../../raws/spawns.json");

#[derive(Deserialize, Debug)]
pub struct Raws {
    pub items: Vec<item_structs::Item>,
    pub mobs: Vec<mob_structs::Mob>,
    pub props: Vec<prop_structs::Prop>,
}

#[derive(Deserialize, Debug)]
pub struct Renderable {
    pub glyph: String,
    pub fg: String,
    pub bg: String,
    pub order: i32,
}

pub fn load_raws() {
    link_resource!(RAW_FILE, "../../raws/spawns.json");

    let raw_data = embedding::EMBED
        .lock()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();
    let raw_string =
        std::str::from_utf8(&raw_data).expect("Unable to convert to valid UTF-8 string");
    let decoder: Raws = serde_json::from_str(&raw_string).expect("Unable to parse JSON");

    RAWS.lock().unwrap().load(decoder);
}
