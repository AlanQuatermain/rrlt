use std::collections::HashSet;
use crate::prelude::*;

#[system]
#[read_component(Ranged)]
#[read_component(Damage)]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(Point)]
#[read_component(AreaOfEffect)]
pub fn ranged_target(
    ecs: &SubWorld,
    #[resource] map: &Map,
    #[resource] turn_state: &mut TurnState,
    #[resource] mouse_pos: &Point,
    #[resource] mouse_clicked: &bool,
    #[resource] key: &Option<VirtualKeyCode>,
    commands: &mut CommandBuffer
) {
    let (range, item_entity) = match *turn_state {
        TurnState::RangedTargeting { range, item } => (range, item),
        _ => return
    };
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let player_fov = fov.iter(ecs).nth(0).unwrap();
    let (player_pos, _, player) = <(&Point, &Player, Entity)>::query().iter(ecs).nth(0).unwrap();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);

    draw_batch.print_color(Point::new(5, 0),
                           "Select Target",
                           ColorPair::new(YELLOW, BLACK));

    let mut available_cells = HashSet::new();
    for pos in &player_fov.visible_tiles {
        let distance = DistanceAlg::Pythagoras.distance2d(*player_pos, *pos);
        if distance <= range as f32 {
            draw_batch.set_bg(*pos, BLUE);
            available_cells.insert(pos);
        }
    }

    let mut radius = 1;
    if let Ok(area_of_effect) = ecs.entry_ref(item_entity).unwrap().get_component::<AreaOfEffect>() {
        radius = area_of_effect.0;
    }

    // Draw mouse cursor
    if available_cells.contains(mouse_pos) {
        if radius <= 1 {
            draw_batch.set_bg(*mouse_pos, CYAN);
        }
        else {
            let tiles = field_of_view_set(*mouse_pos, radius, map);
            for pos in tiles {
                draw_batch.set_bg(pos, CYAN);
            }
        }
        if *mouse_clicked {
            commands.push(((), ActivateItem {
                used_by: *player,
                item: item_entity,
                target: Some(*mouse_pos)
            }));
            *turn_state = TurnState::PlayerTurn;
            return;
        }
    }

    draw_batch.submit(2000).expect("Batch error");

    if let Some(key) = *key {
        match key {
            VirtualKeyCode::Escape => *turn_state = TurnState::ShowingInventory,
            _ => {}
        }
    }
}