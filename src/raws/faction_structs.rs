use crate::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct FactionInfo {
    pub name: String,
    pub responses: HashMap<String, String>,
}

#[derive(PartialEq, Debug, Hash, Eq, Copy, Clone)]
pub enum Reaction {
    Ignore,
    Attack,
    Flee,
}
