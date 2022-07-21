use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LootTable {
    pub name: String,
    pub drops: Vec<LootDrop>,
}

#[derive(Debug, Deserialize)]
pub struct LootDrop {
    pub name: String,
    pub weight: i32,
}
