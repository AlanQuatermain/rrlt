use crate::prelude::*;

pub fn level_transition(
    ecs: &mut World,
    resources: &mut Resources,
    rng: &mut RandomNumberGenerator,
    new_depth: i32,
    offset: i32,
) -> Option<Vec<Map>> {
    // Obtain the master dungeon map from the resources.
    let dungeon_master = resources.get_or_default::<MasterDungeonMap>();

    // Do we already have a map?
    if dungeon_master.get_map(new_depth).is_some() {
        std::mem::drop(dungeon_master);
        transition_to_existing_map(ecs, resources, new_depth, offset);
        None
    } else {
        std::mem::drop(dungeon_master);
        Some(transition_to_new_map(ecs, resources, rng, new_depth))
    }
}

fn transition_to_new_map(
    ecs: &mut World,
    resources: &mut Resources,
    rng: &mut RandomNumberGenerator,
    new_depth: i32,
) -> Vec<Map> {
    let mut builder = level_builder(new_depth, 80, 50, rng);
    builder.build_map(rng);

    let dm = resources.get::<MasterDungeonMap>().unwrap();
    builder.spawn_entities(ecs, &dm);
    std::mem::drop(dm);

    if let Some(pos) = &builder.build_data.starting_position {
        if new_depth != 0 {
            let up_idx = builder.build_data.map.point2d_to_index(*pos);
            builder.build_data.map.tiles[up_idx] = TileType::UpStairs;
        }

        // Update player if they exist.
        let mut found_user = false;
        <(&mut Point, &mut FieldOfView)>::query()
            .filter(component::<Player>())
            .for_each_mut(ecs, |(pt, fov)| {
                *pt = *pos;
                fov.is_dirty = true;
                found_user = true;
            });

        if !found_user {
            // Need to spawn the player
            let dm = resources.get::<MasterDungeonMap>().unwrap();
            spawn_player(ecs, &dm, *pos);
        }

        // Update the camera
        resources.insert(Camera::new(*pos));
    }

    // Put the map into resources *after* we add the up-stairs
    resources.insert(builder.build_data.map.clone());

    // Store in the dungeon master
    let mut dungeon_master = resources.get_mut::<MasterDungeonMap>().unwrap();
    dungeon_master.store_map(&builder.build_data.map);

    builder.build_data.history
}

fn transition_to_existing_map(
    ecs: &mut World,
    resources: &mut Resources,
    new_depth: i32,
    offset: i32,
) {
    // We know it's here at this point.
    let dungeon_master = resources.get::<MasterDungeonMap>().unwrap();
    let map = dungeon_master.get_map(new_depth).unwrap();
    std::mem::drop(dungeon_master);

    resources.insert(map.clone());

    // Find the appropriate stairs and place the player there.
    let stair_type = if offset < 0 {
        TileType::DownStairs
    } else {
        TileType::UpStairs
    };

    for (idx, tt) in map.tiles.iter().enumerate() {
        if *tt == stair_type {
            let pos = map.index_to_point2d(idx);
            <(&mut Point, &mut FieldOfView)>::query()
                .filter(component::<Player>())
                .for_each_mut(ecs, |(pt, fov)| {
                    *pt = pos;
                    fov.is_dirty = true;
                    resources.insert(Camera::new(pos));
                });
            break;
        }
    }
}
