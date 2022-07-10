use crate::prelude::*;

#[system]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(Health)]
pub fn gui(ecs: &SubWorld, #[resource] gamelog: &Gamelog, #[resource] turn_state: &mut TurnState) {
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);
    draw_batch.draw_box(
        Rect::with_size(0, MAP_HEIGHT, 79, 6),
        ColorPair::new(WHITE, BLACK));

    let player = <&Player>::query()
        .iter(ecs).nth(0).unwrap();
    let depth = format!("Depth: {}", player.map_level+1);
    draw_batch.print_color(
        Point::new(2, 43),
        depth,
        ColorPair::new(YELLOW, BLACK)
    );

    let mut health_query = <&Health>::query()
        .filter(component::<Player>());
    let player_health = health_query
        .iter(ecs)
        .nth(0)
        .unwrap();
    let health = format!(" HP: {} / {}", player_health.current, player_health.max);
    draw_batch.print_color(
        Point::new(12, 43),
        health,
        ColorPair::new(YELLOW, BLACK));
    draw_batch.bar_horizontal(
        Point::new(28, 43),
        51,
        player_health.current,
        player_health.max,
        ColorPair::new(RED, BLACK));

    let mut y = 44;
    for s in gamelog.entries.iter().rev() {
        if y < 49 { draw_batch.print(Point::new(2, y), s); }
        y += 1;
    }

    draw_batch.submit(5000).expect("Batch error");
}