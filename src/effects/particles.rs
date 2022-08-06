use crate::prelude::*;

pub fn particle_to_tile(
    _ecs: &mut SubWorld,
    indices: Vec<usize>,
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
        for tile_idx in indices {
            let pos = map.index_to_point2d(tile_idx);
            particle_builder.request(pos, color, glyph, lifespan);
        }
    }
}

pub fn projectile(
    _ecs: &mut SubWorld,
    tile_idx: usize,
    effect: &EffectSpawner,
    map: &Map,
    particle_builder: &mut ParticleBuilder,
) {
    if let EffectType::ParticleProjectile {
        glyph,
        color,
        lifespan: _,
        speed,
        path,
    } = &effect.effect_type
    {
        let pos = map.index_to_point2d(tile_idx);
        particle_builder.request_animated(pos, *color, *glyph, *speed, path.to_vec());
    }
}
