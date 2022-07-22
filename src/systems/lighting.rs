use crate::prelude::*;

#[system]
#[read_component(LightSource)]
#[read_component(Point)]
#[read_component(FieldOfView)]
pub fn lighting(ecs: &SubWorld, #[resource] map: &mut Map, commands: &mut CommandBuffer) {
    if map.outdoors {
        return;
    }

    let black = RGB::named(BLACK);
    for l in map.light.iter_mut() {
        *l = black;
    }

    <(&Point, &FieldOfView, &LightSource)>::query().for_each(ecs, |(pos, fov, light)| {
        let light_point = *pos;
        let range_f = light.range as f32;
        for t in fov.visible_tiles.iter() {
            if map.in_bounds(*t) {
                let idx = map.point2d_to_index(*t);
                let distance = DistanceAlg::Pythagoras.distance2d(light_point, *t);
                let intensity = (range_f - distance) / range_f;

                map.light[idx] = map.light[idx] + (light.color * intensity);
            }
        }
    });
}
