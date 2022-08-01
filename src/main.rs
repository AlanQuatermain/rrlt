mod camera;
mod components;
mod effects;
mod gamelog;
mod gamesystem;
mod map;
mod map_builder;
mod random_table;
mod raws;
mod rex_assets;
mod spatial;
mod spawner;
mod systems;
mod turn_state;

#[allow(dead_code)]
mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::serialize::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;
    pub use serde::*;

    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 60;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;

    pub const SHOW_MAPGEN_VISUALIZER: bool = false;
    pub const SHOW_BOUNDARIES: bool = false;

    pub const FINAL_LEVEL: u32 = 2;

    pub use crate::camera::*;
    pub use crate::components::*;
    pub use crate::effects::*;
    pub use crate::gamelog::*;
    pub use crate::gamesystem::*;
    pub use crate::map::*;
    pub use crate::map_builder::*;
    pub use crate::random_table::*;
    pub use crate::raws::*;
    pub use crate::rex_assets::*;
    pub use crate::spawner::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;
}

use legion::serialize::UnknownType::Ignore;
use prelude::*;
use std::fs;
use std::fs::File;

#[macro_use]
extern crate lazy_static;

pub struct KeyState {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub mouse_pos: Point,
    pub mouse_clicked: bool,
    pub key: Option<VirtualKeyCode>,
}

impl KeyState {
    fn new(ctx: &BTerm) -> Self {
        Self {
            shift: ctx.shift,
            control: ctx.control,
            alt: ctx.alt,
            mouse_pos: Point::from_tuple(ctx.mouse_pos()),
            mouse_clicked: ctx.left_click,
            key: ctx.key,
        }
    }
}

struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    tick_systems: Schedule,
    ranged_systems: Schedule,
    menu_systems: Schedule,
    popup_menu_systems: Schedule,

    map_history: Vec<Map>,
    real_map: Map,
    mapgen_timer: f32,
}

impl State {
    fn new() -> Self {
        let ecs = World::default();
        let mut resources = Resources::default();

        resources.insert(TurnState::MainMenu {
            selection: MainMenuSelection::NewGame,
        });
        resources.insert(RexAssets::new());
        resources.insert(MasterDungeonMap::new());

        Self {
            ecs,
            resources,
            input_systems: build_input_scheduler(),
            tick_systems: build_ticking_scheduler(),
            ranged_systems: build_ranged_scheduler(),
            menu_systems: build_menu_scheduler(),
            popup_menu_systems: build_popup_scheduler(),
            map_history: Vec::default(),
            real_map: Map::default(),
            mapgen_timer: 0.0,
        }
    }

    fn make_new_game(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        self.resources.insert(RexAssets::new());
        self.resources.insert(MasterDungeonMap::new());

        let mut rng = RandomNumberGenerator::new();
        self.conjure_map(&mut rng, 0, 0);

        let mut gamelog = Gamelog::default();
        gamelog
            .entries
            .push("Welcome to Rusty Roguelike".to_string());

        self.resources.insert(rng);
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(gamelog);

        if SHOW_MAPGEN_VISUALIZER {
            self.resources.insert(TurnState::MapBuilding { step: 0 });
        }
    }

    fn conjure_map(&mut self, rng: &mut RandomNumberGenerator, new_depth: i32, offset: i32) {
        if SHOW_MAPGEN_VISUALIZER {
            self.mapgen_timer = 0.0;
            self.map_history.clear();
        }

        let map_building_info =
            map::level_transition(&mut self.ecs, &mut self.resources, rng, new_depth, offset);
        if let Some(history) = map_building_info {
            if SHOW_MAPGEN_VISUALIZER {
                self.map_history = history;
            }
        } else {
            let mut cb = CommandBuffer::new(&self.ecs);
            thaw_level_entities(&self.ecs, new_depth, &mut cb);
            cb.flush(&mut self.ecs, &mut self.resources);
        }
    }

    fn switch_level(&mut self, offset: i32) {
        let current_map = self.resources.get::<Map>().unwrap().clone();
        let map_level = current_map.depth;
        let new_depth = map_level + offset;

        // Save the full current state of the map in the master
        let mut dungeon_master = self.resources.get_mut::<MasterDungeonMap>().unwrap();
        dungeon_master.store_map(&current_map);
        std::mem::drop(dungeon_master);

        let mut cb = CommandBuffer::new(&mut self.ecs);
        freeze_level_entities(&self.ecs, map_level, &mut cb);
        cb.flush(&mut self.ecs, &mut self.resources);

        let mut rng = RandomNumberGenerator::new();
        self.conjure_map(&mut rng, new_depth, offset);

        self.resources.insert(rng);
        self.resources.insert(TurnState::AwaitingInput);

        <&mut FieldOfView>::query()
            .filter(!component::<OtherLevelPosition>())
            .iter_mut(&mut self.ecs)
            .for_each(|fov| fov.is_dirty = true);

        if SHOW_MAPGEN_VISUALIZER {
            self.resources.insert(TurnState::MapBuilding { step: 0 });
        }

        self.resources
            .get_mut::<Gamelog>()
            .unwrap()
            .entries
            .push("You descend to the next level.".to_string());
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(2);

        ctx.print_color_centered(15, YELLOW, BLACK, "Your journey has ended!");

        ctx.print_color_centered(
            17,
            WHITE,
            BLACK,
            "One day, we'll tell you all about how you did.",
        );
        ctx.print_color_centered(
            18,
            WHITE,
            BLACK,
            "That day, sadly, is not in this chapter...",
        );

        ctx.print_color_centered(20, MAGENTA, BLACK, "Press any key to return to the menu.");

        if ctx.key.is_some() {
            ctx.key = None;
            self.resources.insert(TurnState::MainMenu {
                selection: MainMenuSelection::NewGame,
            });
        }
    }

    fn configure_registry(&self, registry: &mut Registry<String>) {
        registry.register::<Point>("position".to_string());
        registry.register::<Render>("render".to_string());
        registry.register::<Player>("player".to_string());
        registry.register::<Name>("name".to_string());
        registry.register::<Item>("item".to_string());
        registry.register::<AmuletOfYala>("amulet_of_yala".to_string());
        registry.register::<FieldOfView>("fov".to_string());
        registry.register::<ProvidesHealing>("provides_healing".to_string());
        registry.register::<ProvidesDungeonMap>("provides_map".to_string());
        registry.register::<Carried>("carried_by".to_string());
        registry.register::<Damage>("damage".to_string());
        registry.register::<WeaponAttribute>("wattr".to_string());
        registry.register::<MeleeWeapon>("melee_weapon".to_string());
        registry.register::<Wearable>("wearable".to_string());
        registry.register::<BlocksTile>("blocks_tile".to_string());
        registry.register::<Consumable>("consumable".to_string());
        registry.register::<Ranged>("ranged".to_string());
        registry.register::<AreaOfEffect>("aoe".to_string());
        registry.register::<Confusion>("confusion".to_string());
        registry.register::<Map>("map".to_string());
        registry.register::<MapTheme>("theme".to_string());
        registry.register::<SerializeMe>("serialize".to_string());
        registry.register::<EquipmentSlot>("slot".to_string());
        registry.register::<Equippable>("equippable".to_string());
        registry.register::<Equipped>("equipped".to_string());
        registry.register::<ParticleLifetime>("particle_lifetime".to_string());
        registry.register::<HungerState>("hunger_state".to_string());
        registry.register::<HungerClock>("hunger_clock".to_string());
        registry.register::<ProvidesFood>("provides_food".to_string());
        registry.register::<Hidden>("hidden".to_string());
        registry.register::<EntryTrigger>("entry_trigger".to_string());
        registry.register::<SingleActivation>("one_shot".to_string());
        registry.register::<BlocksVisibility>("blocks_visibility".to_string());
        registry.register::<Door>("door".to_string());
        registry.register::<AlwaysVisible>("always_visible".to_string());
        registry.register::<Quips>("quips".to_string());
        registry.register::<Attribute>("attr".to_string());
        registry.register::<Attributes>("attrs".to_string());
        registry.register::<Skill>("skill".to_string());
        registry.register::<Skills>("skills".to_string());
        registry.register::<Pool>("pool".to_string());
        registry.register::<Pools>("pools".to_string());
        registry.register::<NaturalAttack>("nattack".to_string());
        registry.register::<NaturalAttackDefense>("natkdef".to_string());
        registry.register::<LootTable>("loot_tbl".to_string());
        registry.register::<OtherLevelPosition>("olpos".to_string());
        registry.register::<LightSource>("light_source".to_string());
        registry.register::<Initiative>("initiative".to_string());
        registry.register::<MyTurn>("my_turn".to_string());
        registry.register::<Faction>("faction".to_string());
        registry.register::<Movement>("movement".to_string());
        registry.register::<MoveMode>("move_mode".to_string());
        registry.register::<Chasing>("chasing".to_string());
        registry.register::<Vendor>("vendor".to_string());
        registry.register::<TownPortal>("town_portal".to_string());
        registry.register::<MagicItemClass>("magic_item_class".to_string());
        registry.register::<MagicItem>("magic_item".to_string());
        registry.register::<ObfuscatedName>("obf_name".to_string());
        registry.register::<IdentifiedItem>("identified_item".to_string());
        registry.register::<MasterDungeonMap>("dungeon_master".to_string());
        registry.register::<CursedItem>("cursed".to_string());
        registry.register::<ProvidesRemoveCurse>("removes_curse".to_string());
        registry.register::<ProvidesIdentify>("identifies".to_string());
        registry.on_unknown(Ignore);
    }

    fn save_game(&mut self) {
        let mut registry = Registry::<String>::default();
        self.configure_registry(&mut registry);

        // Temporarily add the map & master to get them included
        let map_entity = self
            .ecs
            .push((self.resources.get::<Map>().unwrap().clone(), SerializeMe));
        let dm_entity = self.ecs.push((
            self.resources.get::<MasterDungeonMap>().unwrap().clone(),
            SerializeMe,
        ));

        let writer = File::create("./savegame.json").unwrap();
        let entity_serializer = Canon::default();
        serde_json::to_writer(
            writer,
            &self
                .ecs
                .as_serializable(component::<SerializeMe>(), &registry, &entity_serializer),
        )
        .expect("Error saving game");

        // remove the map now
        self.ecs.remove(map_entity);
        self.ecs.remove(dm_entity);

        // Show the main menu.
        self.resources.insert(TurnState::MainMenu {
            selection: MainMenuSelection::LoadGame,
        });
    }

    fn load_game(&mut self) {
        use serde::de::DeserializeSeed;

        let mut registry = Registry::new();
        self.configure_registry(&mut registry);
        let entity_serializer = Canon::default();

        let text = fs::read_to_string("./savegame.json").unwrap();
        let json = serde_json::from_str::<serde_json::Value>(text.as_str()).unwrap();
        self.ecs = registry
            .as_deserialize(&entity_serializer)
            .deserialize(json)
            .unwrap();
        self.resources = Resources::default();

        // extract the map and master
        let mut to_remove: Vec<Entity> = Vec::new();
        {
            let (map, map_entity) = <(&Map, Entity)>::query()
                .iter(&mut self.ecs)
                .nth(0)
                .unwrap();
            self.resources.insert(map.clone());
            spatial::set_size(map.width * map.height);
            to_remove.push(*map_entity);

            let (dm, dm_entity) = <(&MasterDungeonMap, Entity)>::query()
                .iter(&mut self.ecs)
                .nth(0)
                .unwrap();
            self.resources.insert(dm.clone());
            to_remove.push(*dm_entity);
        }

        for entity in to_remove {
            self.ecs.remove(entity);
        }

        // build the camera, centered on the player
        let player_pos = <&Point>::query()
            .filter(component::<Player>())
            .iter(&mut self.ecs)
            .nth(0)
            .unwrap();

        let mut gamelog = Gamelog::default();
        gamelog.entries.push("Loaded game.".to_string());

        let mut rng = RandomNumberGenerator::new();

        self.resources.insert(rng);
        self.resources.insert(Camera::new(*player_pos));
        self.resources.insert(TurnState::Ticking);
        self.resources.insert(gamelog);
        self.resources.insert(RexAssets::new());

        // make all FOVs dirty
        <&mut FieldOfView>::query().for_each_mut(&mut self.ecs, |mut fov| {
            fov.is_dirty = true;
        });

        // Permadeath: DELETE THE SAVED GAME!
        fs::remove_file("./savegame.json").expect("Save deletion failed");
    }

    fn reveal_map(&mut self, row: i32) {
        let height: usize;
        {
            let mut map = self.resources.get_mut::<Map>().unwrap();
            for x in 0..map.width {
                let idx = map.point2d_to_index(Point::new(x, row as usize));
                map.revealed_tiles[idx] = true;
            }
            height = map.height;
        }
        if row as usize == height - 1 {
            self.resources.insert(TurnState::Ticking);
        } else {
            self.resources.insert(TurnState::RevealMap { row: row + 1 })
        }
        map_reveal_scheduler().execute(&mut self.ecs, &mut self.resources);
    }

    fn visualize_map_build(&mut self, step: usize, ctx: &BTerm) {
        let mut continue_build = false;

        if SHOW_MAPGEN_VISUALIZER {
            self.mapgen_timer += ctx.frame_time_ms;

            if step < self.map_history.len() {
                self.resources.insert(self.map_history[step].clone());
                continue_build = true;
            } else if ctx.key.is_none() {
                // wait for user to press a key
                continue_build = true;
            }
        }

        if continue_build {
            map_reveal_scheduler().execute(&mut self.ecs, &mut self.resources);

            self.mapgen_timer += ctx.frame_time_ms;
            if self.mapgen_timer < 300.0 {
                return;
            }
            self.mapgen_timer = 0.0;
            self.resources
                .insert(TurnState::MapBuilding { step: step + 1 });
        } else {
            self.map_history.clear();
            self.resources.insert(self.real_map.clone());
            self.resources.insert(TurnState::AwaitingInput);
        }
    }
}

#[allow(dead_code)]
impl GameState for State {
    #[allow(dead_code)]
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();

        ctx.set_active_console(0);
        self.resources.insert(KeyState::new(ctx));
        self.resources.insert(ctx.frame_time_ms);
        self.resources.insert(ParticleBuilder::new());

        // ctx.key = None;

        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::AwaitingInput => {
                self.input_systems
                    .execute(&mut self.ecs, &mut self.resources);
                if self.resources.get::<TurnState>().unwrap().clone() != current_state {
                    // if we changed state, clear keyboard input
                    ctx.key = None;
                }
            }
            TurnState::Ticking => {
                while self.resources.get::<TurnState>().unwrap().clone() == TurnState::Ticking {
                    self.tick_systems
                        .execute(&mut self.ecs, &mut self.resources);
                }
            }
            TurnState::ShowingInventory
            | TurnState::ShowingDropItems
            | TurnState::ShowingRemoveCurse
            | TurnState::ShowingIdentify => self
                .popup_menu_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::RangedTargeting { range: _, item: _ } => self
                .ranged_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::MainMenu { selection: _ } => {
                {
                    // Alas, there's no draw batch command to render a sprite.
                    let assets = self.resources.get::<RexAssets>().unwrap();
                    ctx.render_xp_sprite(&assets.menu, 0, 0);
                }
                self.menu_systems
                    .execute(&mut self.ecs, &mut self.resources)
            }
            TurnState::NewGame => self.make_new_game(),
            TurnState::SaveGame => self.save_game(),
            TurnState::LoadGame => self.load_game(),
            TurnState::NextLevel => self.switch_level(1),
            TurnState::PreviousLevel => self.switch_level(-1),
            TurnState::GameOver => self.game_over(ctx),
            TurnState::RevealMap { row } => self.reveal_map(row),
            TurnState::MapBuilding { step } => self.visualize_map_build(step, ctx),
            TurnState::ShowCheatMenu => {
                build_cheat_menu_scheduler().execute(&mut self.ecs, &mut self.resources)
            }
            TurnState::ShowingVendor { vendor: _, mode: _ } => self
                .input_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::TownPortal => {
                spawn_town_portal(&mut self.ecs, &mut self.resources);
                let map_depth = self.resources.get::<Map>().unwrap().depth;
                self.switch_level(-map_depth);
            }
            TurnState::LevelTeleport { destination, depth } => {
                let map_depth = self.resources.get::<Map>().unwrap().depth;
                self.switch_level(depth - map_depth);
                <&mut Point>::query()
                    .filter(component::<Player>())
                    .iter_mut(&mut self.ecs)
                    .for_each(|pt| *pt = destination);
                self.resources
                    .get_mut::<Camera>()
                    .unwrap()
                    .on_player_move(destination);
            }
        }
        // println!("Tick took {} seconds", tm.elapsed().as_secs_f32());

        render_draw_buffer(ctx).expect("Render error");
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_title("Roguelike Tutorial")
        .with_fps_cap(30.0)
        .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
        // .with_tile_dimensions(32, 32)
        .with_tile_dimensions(16, 16)
        .with_resource_path("resources/")
        // .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png", 8, 8)
        // .with_font("drake_10x10.png", 10, 10)
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .build()?;
    // context.with_post_scanlines(true);

    // let context = BTermBuilder::new()
    //     .with_title("Rouguelike Tutorial")
    //     .with_fps_cap(30.0)
    //     .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
    //     .with_font("terminal8x8.png", 8, 8)
    //     .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
    //     .build()?;

    load_raws();
    main_loop(context, State::new())
}
