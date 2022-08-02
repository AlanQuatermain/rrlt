use crate::prelude::*;

pub fn particle_to_tile(
    _ecs: &mut SubWorld,
    tile_idx: usize,
    effect: &EffectSpawner,
    map: &Map,
    particle_builder: &mut ParticleBuilder,
) {
    if let EffectType::Particle {
        glyph,
        color,
        lifespan,
    } = effect.effect_type
    {
        let pos = map.index_to_point2d(tile_idx);
        particle_builder.request(pos, color, glyph, lifespan);
    }
}
