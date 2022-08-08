use std::collections::HashMap;

use crate::prelude::*;

#[derive(Deserialize, Debug)]
pub struct WeaponTrait {
    pub name: String,
    pub target: Option<String>,
    pub effects: HashMap<String, String>,
}
