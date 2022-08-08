use crate::prelude::*;

#[system(for_each)]
#[read_component(Player)]
#[read_component(Point)]
#[filter(component::<Point>())]
pub fn quipping(
    ecs: &SubWorld,
    fov: &FieldOfView,
    name: &Name,
    quips: &mut Quips,
    #[resource] rng: &mut RandomNumberGenerator,
) {
    let player_pos = <&Point>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .unwrap();
    if !quips.0.is_empty() && fov.visible_tiles.contains(player_pos) && rng.roll_dice(1, 10) == 1 {
        let quip_idx = rng.random_slice_index(quips.0.as_slice()).unwrap();
        crate::gamelog::Logger::new()
            .color(YELLOW)
            .append(&name.0)
            .color(WHITE)
            .append("says")
            .color(CYAN)
            .append(&quips.0[quip_idx])
            .log();
        quips.0.remove(quip_idx);
    }
}
