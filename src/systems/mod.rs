mod animal_ai;
mod bury_dead;
mod bystander_ai;
mod chasing;
mod collect;
mod combat;
mod damage;
mod drop_item;
mod end_turn;
mod entity_render;
mod fov;
mod gui;
mod hunger;
mod inventory;
mod map_indexing;
mod map_render;
mod menu;
mod movement;
mod particles;
mod player_input;
mod ranged_target;
mod tooltips;
mod trigger;
mod use_items;

use crate::prelude::*;

pub use menu::MainMenuSelection;
pub use particles::ParticleBuilder;

pub fn build_input_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(particles::particle_cull_system())
        .add_system(player_input::player_input_system())
        .add_system(collect::collect_system())
        .flush()
        .add_system(fov::fov_system())
        .add_system(particles::particle_spawn_system())
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
        .add_system(particles::particle_cull_system())
        .add_system(use_items::use_items_system())
        .add_system(drop_item::drop_item_system())
        .add_system(combat::combat_system())
        .add_system(hunger::hunger_system())
        .flush()
        .add_system(damage::damage_system())
        .add_system(particles::particle_spawn_system())
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(fov::fov_system())
        .flush()
        .add_system(trigger::trigger_system())
        .flush()
        .add_system(damage::damage_system())
        .add_system(map_indexing::map_indexing_system())
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .add_system(bury_dead::bury_dead_system())
        .add_system(end_turn::end_turn_system())
        .build()
}

pub fn build_monster_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(particles::particle_cull_system())
        .add_system(chasing::chasing_system())
        .add_system(bystander_ai::bystander_ai_system())
        .add_system(animal_ai::animal_ai_system())
        .flush()
        .add_system(use_items::use_items_system())
        .add_system(drop_item::drop_item_system())
        .add_system(combat::combat_system())
        .flush()
        .add_system(hunger::hunger_system())
        .add_system(damage::damage_system())
        .add_system(particles::particle_spawn_system())
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(fov::fov_system())
        .add_system(trigger::trigger_system())
        .flush()
        .add_system(damage::damage_system())
        .add_system(map_indexing::map_indexing_system())
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .add_system(bury_dead::bury_dead_system())
        .add_system(end_turn::end_turn_system())
        .build()
}

pub fn build_ranged_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(particles::particle_cull_system())
        .add_system(ranged_target::ranged_target_system())
        .add_system(fov::fov_system())
        .add_system(particles::particle_spawn_system())
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
        .add_system(particles::particle_cull_system())
        .add_system(particles::particle_spawn_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .add_system(inventory::inventory_system())
        .build()
}

pub fn map_reveal_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .build()
}

fn name_for(entity: &Entity, ecs: &SubWorld) -> (String, bool) {
    if let Ok(name) = ecs.entry_ref(*entity).unwrap().get_component::<Name>() {
        (name.0.clone(), false)
    } else if ecs
        .entry_ref(*entity)
        .unwrap()
        .get_component::<Player>()
        .is_ok()
    {
        ("Hero".to_string(), true)
    } else {
        ("Someone".to_string(), false)
    }
}
