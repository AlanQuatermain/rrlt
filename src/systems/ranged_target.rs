use crate::{prelude::*, KeyState};
use std::collections::HashSet;

#[system]
#[read_component(Ranged)]
#[read_component(Damage)]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(Point)]
#[read_component(AreaOfEffect)]
#[read_component(SpellTemplate)]
pub fn ranged_target(
    ecs: &SubWorld,
    #[resource] map: &Map,
    #[resource] turn_state: &mut TurnState,
    #[resource] key_state: &KeyState,
    #[resource] camera: &Camera,
    commands: &mut CommandBuffer,
) {
    let (range, item_entity) = match *turn_state {
        TurnState::RangedTargeting { range, item } => (range, item),
        _ => return,
    };
    let offset = Point::new(camera.left_x, camera.top_y);
    let map_pos = key_state.mouse_pos + offset;
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let player_fov = fov.iter(ecs).nth(0).unwrap();
    let (player_pos, player) = <(&Point, Entity)>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);

    draw_batch.print_color(
        Point::new(5, 0),
        "Select Target",
        ColorPair::new(YELLOW, BLACK),
    );

    let mut available_cells = HashSet::new();
    for pos in &player_fov.visible_tiles {
        let distance = DistanceAlg::Pythagoras.distance2d(*player_pos, *pos);
        if distance <= range as f32 && !map.tile_matches(pos, TileType::Wall) {
            draw_batch.set_bg(*pos - offset, BLUE);
            available_cells.insert(pos);
        }
    }

    let mut radius = 1;
    if let Ok(area_of_effect) = ecs
        .entry_ref(item_entity)
        .unwrap()
        .get_component::<AreaOfEffect>()
    {
        radius = area_of_effect.0;
    }

    // Draw mouse cursor
    if available_cells.contains(&map_pos) {
        if radius <= 1 {
            draw_batch.set_bg(key_state.mouse_pos, CYAN);
        } else {
            let tiles = field_of_view_set(map_pos, radius, map);
            for pos in tiles {
                if !map.tile_matches(&pos, TileType::Wall) {
                    draw_batch.set_bg(pos - offset, CYAN);
                }
            }
        }
        if key_state.mouse_clicked {
            let entry = ecs.entry_ref(item_entity).unwrap();
            if entry.get_component::<SpellTemplate>().is_ok() {
                commands.add_component(
                    *player,
                    WantsToCastSpell {
                        spell: item_entity,
                        target: Some(map_pos),
                    },
                )
            } else {
                commands.add_component(
                    item_entity,
                    UseItem {
                        user: *player,
                        target: Some(map_pos),
                    },
                );
            }
            *turn_state = TurnState::Ticking;
            return;
        }
    }

    draw_batch.submit(2000).expect("Batch error");

    if let Some(key) = key_state.key {
        match key {
            VirtualKeyCode::Escape => *turn_state = TurnState::ShowingInventory,
            _ => {}
        }
    }
}
