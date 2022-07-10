mod player_input;
mod map_render;
mod movement;
mod entity_render;
mod fov;
mod end_turn;
mod chasing;
mod combat;
mod map_indexing;
mod gui;
mod tooltips;
mod collect;
mod inventory;
mod use_items;
mod drop_item;
mod ranged_target;
mod menu;

use crate::prelude::*;

pub use menu::MainMenuSelection;

pub fn build_input_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(player_input::player_input_system())
        .add_system(collect::collect_system())
        .add_system(fov::fov_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(map_indexing::map_indexing_system())
        .add_system(gui::gui_system())
        .add_system(tooltips::tooltips_system())
        .add_system(inventory::inventory_system())
        .build()
}

pub fn build_player_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(use_items::use_items_system())
        .add_system(drop_item::drop_item_system())
        .add_system(combat::combat_system())
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(fov::fov_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(map_indexing::map_indexing_system())
        .add_system(gui::gui_system())
        .add_system(end_turn::end_turn_system())
        .build()
}

pub fn build_monster_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(chasing::chasing_system())
        .flush()
        .add_system(use_items::use_items_system())
        .add_system(drop_item::drop_item_system())
        .add_system(combat::combat_system())
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(fov::fov_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(map_indexing::map_indexing_system())
        .add_system(gui::gui_system())
        .add_system(end_turn::end_turn_system())
        .build()
}

pub fn build_ranged_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(ranged_target::ranged_target_system())
        .add_system(fov::fov_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(map_indexing::map_indexing_system())
        .add_system(gui::gui_system())
        .add_system(tooltips::tooltips_system())
        .build()
}

pub fn build_menu_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(menu::main_menu_system())
        .build()
}

pub fn build_popup_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .add_system(inventory::inventory_system())
        .build()
}

fn name_for(entity: &Entity, ecs: &SubWorld) -> (String, bool) {
    if let Ok(name) = ecs.entry_ref(*entity).unwrap().get_component::<Name>() {
        (name.0.clone(), false)
    }
    else if ecs.entry_ref(*entity).unwrap().get_component::<Player>().is_ok() {
        ("Player".to_string(), true)
    }
    else {
        ("Someone".to_string(), false)
    }
}