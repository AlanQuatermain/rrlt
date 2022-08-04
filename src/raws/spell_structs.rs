use crate::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Spell {
    pub name: String,
    pub mana_cost: i32,
    pub effects: HashMap<String, String>,
}
