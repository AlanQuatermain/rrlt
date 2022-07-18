use crate::prelude::*;

#[system]
#[read_component(Point)]
#[write_component(ParticleLifetime)]
pub fn particle_cull(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] frame_time_ms: &f32,
) {
    <(Entity, &mut ParticleLifetime)>::query().for_each_mut(ecs, |(entity, lifetime)| {
        lifetime.0 -= frame_time_ms;
        if lifetime.0 <= 0.0 {
            commands.remove(*entity);
        }
    });
}

#[system]
#[read_component(Point)]
pub fn particle_spawn(commands: &mut CommandBuffer, #[resource] builder: &mut ParticleBuilder) {
    for request in &builder.requests {
        commands.push((
            ParticleLifetime(request.lifetime),
            request.pos.clone(),
            Render {
                color: request.color.clone(),
                glyph: request.glyph,
                render_order: 3,
            },
        ));
    }
}

struct ParticleRequest {
    pos: Point,
    color: ColorPair,
    glyph: FontCharType,
    lifetime: f32,
}

#[derive(Default)]
pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {
            requests: Vec::new(),
        }
    }

    pub fn request(&mut self, pos: Point, color: ColorPair, glyph: FontCharType, lifetime: f32) {
        self.requests.push(ParticleRequest {
            pos,
            color,
            glyph,
            lifetime,
        });
    }
}
