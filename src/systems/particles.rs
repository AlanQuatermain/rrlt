use crate::prelude::*;

#[system(for_each)]
#[read_component(Point)]
#[write_component(ParticleLifetime)]
pub fn update(
    entity: &Entity,
    particle: &mut ParticleLifetime,
    pos: &mut Point,
    _ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] frame_time_ms: &f32,
) {
    if let Some(animation) = particle.animation.as_mut() {
        animation.timer += frame_time_ms;
        if animation.timer > animation.step_time
            && animation.current_step < animation.path.len() - 2
        {
            animation.current_step += 1;
            *pos = animation.path[animation.current_step];
        }
    }

    particle.lifetime_ms -= frame_time_ms;
    if particle.lifetime_ms <= 0.0 {
        commands.remove(*entity);
    }
}

#[system]
#[read_component(Point)]
pub fn spawn(commands: &mut CommandBuffer, #[resource] builder: &mut ParticleBuilder) {
    for request in &builder.requests {
        commands.push((
            ParticleLifetime {
                lifetime_ms: request.lifetime,
                animation: request.animation.clone(),
            },
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
    animation: Option<ParticleAnimation>,
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
            animation: None,
        });
    }

    pub fn request_animated(
        &mut self,
        pos: Point,
        color: ColorPair,
        glyph: FontCharType,
        speed: f32,
        path: Vec<Point>,
    ) {
        self.requests.push(ParticleRequest {
            pos,
            color,
            glyph,
            lifetime: speed * path.len() as f32,
            animation: Some(ParticleAnimation {
                step_time: speed,
                path: path.clone(),
                current_step: 0,
                timer: 0.0,
            }),
        });
    }
}
