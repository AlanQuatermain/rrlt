mod ai;
mod bury_dead;
mod collect;
mod combat;
mod damage;
mod drop_item;
mod effects;
mod encumbrance;
mod end_turn;
mod entity_render;
mod fov;
mod gui;
mod hunger;
mod inventory;
mod lighting;
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
mod vendor;

use crate::prelude::*;

pub use ai::*;
pub use menu::MainMenuSelection;
pub use particles::ParticleBuilder;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SystemCondition {
    None,
    RequiresShiftKey,
}

pub fn build_input_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(particles::particle_cull_system())
        .add_system(encumbrance::encumbrance_system())
        .add_system(player_input::player_input_system())
        .add_system(collect::collect_system())
        .flush()
        .add_system(encumbrance::encumbrance_system())
        .add_system(fov::fov_system())
        .add_system(particles::particle_spawn_system())
        .flush()
        .add_system(lighting::lighting_system())
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(map_indexing::map_indexing_system())
        .add_system(gui::gui_system())
        .add_system(tooltips::tooltips_system(SystemCondition::None))
        .add_system(inventory::inventory_system())
        .add_system(vendor::vendor_system())
        .build()
}

pub fn build_ticking_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(particles::particle_cull_system())
        .add_system(map_indexing::map_indexing_system())
        .add_system(fov::fov_system())
        .add_system(encumbrance::encumbrance_system())
        .flush()
        // Determine initiative, assign current turns
        .add_system(ai::initiative::initiative_system())
        .flush()
        // Possible remove current turn based on status effects
        .add_system(ai::turn_status::turn_status_system())
        .flush()
        .add_system(ai::quipping::quipping_system())
        // Immediate responses to nearby entities
        .add_system(ai::adjacent::adjacent_system())
        .flush()
        // Responses to non-adjacent visible entities
        .add_system(ai::visible::visible_system())
        .flush()
        // Per-AI systems to act on determined responses
        .add_system(ai::approach::approach_system())
        .add_system(ai::flee::flee_system())
        .add_system(ai::default::default_movement_system())
        .add_system(use_items::equip_system())
        .flush()
        .add_system(combat::combat_system())
        .add_system(use_items::use_items_system())
        .add_system(use_items::spellcasting_system())
        .add_system(hunger::hunger_system())
        .flush()
        .add_system(damage::damage_system())
        .flush()
        .add_system(drop_item::drop_item_system())
        .add_system(movement::teleport_system()) // may add WantsToMove
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(trigger::trigger_system())
        .flush()
        .add_system(inventory::identification_system())
        .add_system(effects::effects_system())
        .add_system(particles::particle_spawn_system())
        .flush()
        .add_system(lighting::lighting_system())
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .add_system(bury_dead::bury_dead_system())
        // .add_system(end_turn::end_turn_system())
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
        .add_system(tooltips::tooltips_system(SystemCondition::RequiresShiftKey))
        .build()
}

pub fn build_menu_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(menu::main_menu_system())
        .build()
}

pub fn build_cheat_menu_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(particles::particle_cull_system())
        .add_system(lighting::lighting_system())
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .add_system(menu::cheat_menu_system())
        .build()
}

pub fn build_popup_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(particles::particle_cull_system())
        .add_system(particles::particle_spawn_system())
        .flush()
        .add_system(lighting::lighting_system())
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .add_system(inventory::inventory_system())
        .build()
}

pub fn map_reveal_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(lighting::lighting_system())
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(gui::gui_system())
        .build()
}

pub fn name_for(entity: &Entity, ecs: &SubWorld) -> (String, bool) {
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
