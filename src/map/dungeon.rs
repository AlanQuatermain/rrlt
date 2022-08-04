use crate::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps: HashMap<i32, Map>,
    available_scroll_names: Vec<String>,
    available_potion_types: Vec<String>,
    available_wand_types: Vec<String>,

    pub identified_items: HashSet<String>,
    pub scroll_mappings: HashMap<String, String>,
    pub potion_mappings: HashMap<String, String>,
    pub wand_mappings: HashMap<String, String>,
}

impl MasterDungeonMap {
    pub fn new() -> MasterDungeonMap {
        let mut dm = MasterDungeonMap::default();

        dm.build_name_tables();

        let mut rng = RandomNumberGenerator::new();
        for scroll_tag in get_scroll_tags().iter() {
            let idx = rng.random_slice_index(&dm.available_scroll_names).unwrap();
            let masked_name = dm.available_scroll_names.remove(idx);
            dm.scroll_mappings.insert(
                scroll_tag.to_string(),
                format!("Scroll titled {}", masked_name),
            );
        }

        for potion_tag in get_potion_tags().iter() {
            let idx = rng.random_slice_index(&dm.available_potion_types).unwrap();
            let masked_name = dm.available_potion_types.remove(idx);
            dm.potion_mappings
                .insert(potion_tag.to_string(), format!("{} potion", masked_name));
        }

        for wand_tag in get_wand_tags().iter() {
            let idx = rng.random_slice_index(&dm.available_wand_types).unwrap();
            let masked_name = dm.available_wand_types.remove(idx);
            dm.wand_mappings
                .insert(wand_tag.to_string(), format!("{} wand", masked_name));
        }

        dm
    }

    pub fn store_map(&mut self, map: &Map) {
        self.maps.insert(map.depth, map.clone());
    }

    pub fn get_map(&self, depth: i32) -> Option<Map> {
        self.maps.get(&depth).map(|m| m.clone())
    }

    fn build_name_tables(&mut self) {
        self.available_scroll_names = vec![
            "ZELGO MER",
            "JUYED AWK YACC",
            "NR 9",
            "XIXAXA XOXAXA XUXAXA",
            "PRATYAVAYAH",
            "DAIYEN FOOELS",
            "LEP GEX VEN ZEA",
            "PRIRUTSENIE",
            "ELBIB YLOH",
            "VERR YED HORRE",
            "VENZAR BORGAVVE",
            "THARR",
            "YUM YUM",
            "KERNOD WEL",
            "ELAM EBOW",
            "DUAM XNAHT",
            "ANDOVA BEGARIN",
            "KIRJE",
            "VE FORBRYDERNE",
            "HACKEM MUCHE",
            "VELOX NEB",
            "FOOBIE BLETCH",
            "TEMOV",
            "GARVEN DEH",
            "READ ME",
        ]
        .iter()
        .map(|a| a.to_string())
        .collect();
        self.available_potion_types = vec![
            "Ruby",
            "Pink",
            "Orange",
            "Yellow",
            "Emerald",
            "Dark green",
            "Cyan",
            "Sky blue",
            "Brilliant blue",
            "Magenta",
            "Purple-red",
            "Puce",
            "Milky",
            "Swirly",
            "Bubbly",
            "Smoky",
            "Cloudy",
            "Effervescent",
            "Black",
            "Golden",
            "Brown",
            "Fizzy",
            "Dark",
            "White",
            "Murky",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        self.available_wand_types = vec![
            "Glass",
            "Balsa",
            "Crystal",
            "Maple",
            "Pine",
            "Oak",
            "Ebony",
            "Marble",
            "Tin",
            "Brass",
            "Copper",
            "Silver",
            "Platinum",
            "Iridium",
            "Zinc",
            "Aluminum",
            "Uranium",
            "Iron",
            "Steel",
            "Hexagonal",
            "Short",
            "Runed",
            "Long",
            "Curved",
            "Forked",
            "Spiked",
            "Jeweled",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
    }
}

pub fn freeze_level_entities(ecs: &World, depth: i32, commands: &mut CommandBuffer) {
    <(Entity, &Point)>::query()
        .filter(!component::<Player>() & !component::<OtherLevelPosition>())
        .for_each(ecs, |(entity, pos)| {
            commands.add_component(
                *entity,
                OtherLevelPosition {
                    position: pos.clone(),
                    depth,
                },
            );
            commands.remove_component::<Point>(*entity);
        });
}

pub fn thaw_level_entities(ecs: &World, depth: i32, commands: &mut CommandBuffer) {
    <(Entity, &OtherLevelPosition)>::query()
        .iter(ecs)
        .filter(|(_, opos)| opos.depth == depth)
        .for_each(|(entity, opos)| {
            commands.add_component(*entity, opos.position.clone());
            commands.remove_component::<OtherLevelPosition>(*entity);
        });
}
