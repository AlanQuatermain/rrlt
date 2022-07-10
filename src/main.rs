mod components;
mod systems;
mod turn_state;
mod map;
mod camera;
mod map_builder;
mod spawner;
mod gamelog;
mod random_table;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::*;
    pub use legion::serialize::*;
    pub use legion::world::SubWorld;
    pub use legion::systems::CommandBuffer;
    pub use serde::*;

    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;

    pub const FINAL_LEVEL: u32 = 2;

    pub use crate::components::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;
    pub use crate::map::*;
    pub use crate::camera::*;
    pub use crate::map_builder::*;
    pub use crate::spawner::*;
    pub use crate::gamelog::*;
    pub use crate::random_table::*;
}

use std::borrow::Borrow;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use legion::serialize::UnknownType::Ignore;
use serde_json::Value;
use prelude::*;

struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule,
    ranged_systems: Schedule,
    menu_systems: Schedule,
    popup_menu_systems: Schedule,
}

impl State {
    fn new() -> Self {
        let mut ecs = World::default();
        let mut resources = Resources::default();

        resources.insert(TurnState::MainMenu{selection: MainMenuSelection::NewGame});

        Self {
            ecs, resources,
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            monster_systems: build_monster_scheduler(),
            ranged_systems: build_ranged_scheduler(),
            menu_systems: build_menu_scheduler(),
            popup_menu_systems: build_popup_scheduler(),
        }
    }

    fn make_new_game(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();

        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng, 0);
        let mut gamelog = Gamelog::default();
        gamelog.entries.push("Welcome to Rusty Roguelike".to_string());

        spawn_player(&mut self.ecs, map_builder.player_start);
        for pos in map_builder.spawns {
            spawn_mob(&mut self.ecs, pos, &map_builder.random_table, &mut rng);
        }
        let goal_idx = map_builder.map.point2d_to_index(map_builder.goal_start);
        map_builder.map.tiles[goal_idx] = TileType::DownStairs;

        self.resources.insert(rng);
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        self.resources.insert(gamelog);
    }

    fn advance_level(&mut self) {
        let player_entity = *<Entity>::query()
            .filter(component::<Player>())
            .iter(&mut self.ecs)
            .nth(0)
            .unwrap();

        use std::collections::HashSet;
        let mut entities_to_keep = HashSet::new();
        entities_to_keep.insert(player_entity);
        <(Entity, &Carried)>::query()
            .iter(&self.ecs)
            .filter(|(_, carry)| carry.0 == player_entity)
            .map(|(e, _)| *e)
            .for_each(|e| { entities_to_keep.insert(e); });

        let mut cb = CommandBuffer::new(&mut self.ecs);
        for e in Entity::query().iter(&self.ecs) {
            if !entities_to_keep.contains(e) {
                cb.remove(*e);
            }
        }
        cb.flush(&mut self.ecs, &mut self.resources);

        <&mut FieldOfView>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|fov| fov.is_dirty = true);

        let map_level = <&Player>::query()
            .iter(&self.ecs)
            .nth(0)
            .unwrap()
            .map_level as i32 + 1;

        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng, map_level);
        <(&mut Player, &mut Point, &mut Health)>::query()
            .for_each_mut(&mut self.ecs, |(player, pos, health)| {
                player.map_level = map_level as u32;
                *pos = map_builder.player_start;
                health.current = i32::max(health.current, health.max/2);
            });

        let exit_idx = map_builder.map.point2d_to_index(map_builder.goal_start);
        map_builder.map.tiles[exit_idx] = TileType::DownStairs;

        for pos in map_builder.spawns {
            spawn_mob(&mut self.ecs, pos, &map_builder.random_table, &mut rng);
        }

        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);

        self.resources.get_mut::<Gamelog>().unwrap()
            .entries.push("You descend to the next level, taking a moment to heal.".to_string());
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(2);

        ctx.print_color_centered(15, YELLOW, BLACK, "Your journey has ended!");

        ctx.print_color_centered(17, WHITE, BLACK, "One day, we'll tell you all about how you did.");
        ctx.print_color_centered(18, WHITE, BLACK, "That day, sadly, is not int this chapter...");

        ctx.print_color_centered(20, MAGENTA, BLACK, "Press any key to return to the menu.");

        if ctx.key.is_some() {
            ctx.key = None;
            self.resources.insert(TurnState::MainMenu{selection: MainMenuSelection::NewGame});
        }
    }

    fn configure_registry(&self, registry: &mut Registry<String>) {
        registry.register::<Point>("position".to_string());
        registry.register::<Render>("render".to_string());
        registry.register::<Player>("player".to_string());
        registry.register::<Enemy>("enemy".to_string());
        registry.register::<Health>("health".to_string());
        registry.register::<Name>("name".to_string());
        registry.register::<ChasingPlayer>("chasing_player".to_string());
        registry.register::<Item>("item".to_string());
        registry.register::<AmuletOfYala>("amulet_of_yala".to_string());
        registry.register::<ProvidesHealing>("provides_healing".to_string());
        registry.register::<ProvidesDungeonMap>("provides_map".to_string());
        registry.register::<Damage>("damage".to_string());
        registry.register::<Weapon>("weapon".to_string());
        registry.register::<BlocksTile>("blocks_tile".to_string());
        registry.register::<Armor>("armor".to_string());
        registry.register::<Consumable>("consumable".to_string());
        registry.register::<Ranged>("ranged".to_string());
        registry.register::<AreaOfEffect>("aoe".to_string());
        registry.register::<Confusion>("confusion".to_string());
        registry.register::<Map>("map".to_string());
        registry.register::<FieldOfView>("fov".to_string());
        registry.register::<MapTheme>("theme".to_string());
        registry.register::<Carried>("carried_by".to_string());
        registry.register::<SerializeMe>("serialize".to_string());
        registry.register::<EquipmentSlot>("slot".to_string());
        registry.register::<Equippable>("equippable".to_string());
        registry.register::<Equipped>("equipped".to_string());
        registry.on_unknown(Ignore);
    }

    fn save_game(&mut self) {
        let mut registry = Registry::<String>::default();
        self.configure_registry(&mut registry);

        // Temporarily add the map to get it included
        let map_entity = self.ecs.push(
            (
                self.resources.get::<Map>().unwrap().clone(),
                self.resources.get::<MapTheme>().unwrap().clone(),
                SerializeMe,
            )
        );

        let writer = File::create("./savegame.json").unwrap();
        let entity_serializer = Canon::default();
        serde_json::to_writer(writer,&self.ecs.as_serializable(
            component::<SerializeMe>(),
            &registry,
            &entity_serializer
        )).expect("Error saving game");

        // remove the map now
        self.ecs.remove(map_entity);

        // Show the main menu.
        self.resources.insert(TurnState::MainMenu{ selection: MainMenuSelection::LoadGame });
    }

    fn load_game(&mut self) {
        use serde::de::DeserializeSeed;

        let mut registry = Registry::new();
        self.configure_registry(&mut registry);
        let entity_serializer = Canon::default();

        let text = fs::read_to_string("./savegame.json").unwrap();
        let json = serde_json::from_str::<serde_json::Value>(text.as_str()).unwrap();
        println!("{:#?}", json);
        self.ecs = registry
            .as_deserialize(&entity_serializer)
            .deserialize(json)
            .unwrap();
        self.resources = Resources::default();

        // extract the map & theme
        let entity;
        {
            let (map, theme, map_entity) = <(&Map, &MapTheme, Entity)>::query()
                .iter(&mut self.ecs)
                .nth(0)
                .unwrap();
            self.resources.insert(map.clone());
            self.resources.insert(*theme);
            entity = *map_entity;
        }
        self.ecs.remove(entity);

        // build the camera, centered on the player
        let player_pos = <&Point>::query()
            .filter(component::<Player>())
            .iter(&mut self.ecs)
            .nth(0)
            .unwrap();

        let mut rng = RandomNumberGenerator::new();
        let mut gamelog = Gamelog::default();
        gamelog.entries.push("Loaded game.".to_string());

        self.resources.insert(rng);
        self.resources.insert(Camera::new(*player_pos));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(gamelog);

        // make all FOVs dirty
        <&mut FieldOfView>::query()
            .for_each_mut(&mut self.ecs, |mut fov| {
                fov.is_dirty = true;
            });

        // Permadeath: DELETE THE SAVED GAME!
        fs::remove_file("./savegame.json")
            .expect("Save deletion failed");
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();

        self.resources.insert(ctx.key);
        ctx.set_active_console(0);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));
        self.resources.insert(ctx.left_click);

        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::AwaitingInput => self.input_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::PlayerTurn => self.player_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::MonsterTurn => self.monster_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::ShowingInventory => self.popup_menu_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::ShowingDropItems => self.popup_menu_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::RangedTargeting { range, item } => self.ranged_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::MainMenu{selection} => self.menu_systems.execute(&mut self.ecs, &mut self.resources),
            TurnState::NewGame => self.make_new_game(),
            TurnState::SaveGame => self.save_game(),
            TurnState::LoadGame => self.load_game(),
            TurnState::NextLevel => self.advance_level(),
            TurnState::GameOver => self.game_over(ctx)
        }

        render_draw_buffer(ctx)
            .expect("Render error");
    }
}

fn main() -> BError {
    let mut context = BTermBuilder::new()
        .with_title("Roguelike Tutorial")
        .with_fps_cap(30.0)
        .with_dimensions(SCREEN_WIDTH, SCREEN_HEIGHT)
        // .with_tile_dimensions(32, 32)
        .with_tile_dimensions(16, 16)
        .with_resource_path("resources/")
        // .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png", 8, 8)
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .with_simple_console_no_bg(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .with_sparse_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .build()?;
    context.with_post_scanlines(true);

    // let context = BTermBuilder::new()
    //     .with_title("Rouguelike Tutorial")
    //     .with_fps_cap(30.0)
    //     .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
    //     .with_font("terminal8x8.png", 8, 8)
    //     .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
    //     .build()?;

    main_loop(context, State::new())
}
