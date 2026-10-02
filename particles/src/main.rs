mod buffers;
mod effects;
mod modifier;
mod particle;
mod pipelines;
mod system;

use framework::{Config, framework::App, run_app};
use rand::random_range;

use crate::{
    effects::EffectBuilder,
    modifier::{
        Attribute, ConstModifier, InitPositionCircleModifier, LinearDragModifier, ParamModifier,
        Value, VelocityCircleModifier,
    },
    system::ParticleSystem,
};

struct State {
    system: ParticleSystem,
}

impl App for State {
    fn init(render_context: &mut framework::render_context::RenderContext) -> Self {
        let mut system = ParticleSystem::new(render_context, None, 12312313);

        let firework = EffectBuilder::new(
            "fireworks",
            [
                (render_context.surface_size.0 / 2) as f32,
                (render_context.surface_size.1 / 2) as f32,
            ],
            16384,
        )
        .spawn_interval(1.0)
        .particle_size(1)
        .emission_mode(effects::EmissionMode::Burst(1024))
        .init(ConstModifier {
            attribute: Attribute::Lifetime,
            value: Value::RandRange {
                start: 1.0,
                end: 2.0,
            },
        })
        .init(InitPositionCircleModifier {
            radius: Value::RandRange {
                start: 1.0,
                end: 100.0,
            },
        })
        .init(VelocityCircleModifier {
            velocity: Value::RandRange {
                start: 10.0,
                end: 200.0,
            },
            dir: modifier::Direction::Outward,
        })
        .init(ParamModifier {
            param_slot: 0,
            attribute: Attribute::Color,
        })
        .param_update_fn(|mut old_params| {
            old_params[0] = [
                0xFF0000FF, 0xFFFFFF00, 0xFF00FF00, 0xFFFF00FF, 0xFFFFFFFF, 0xFF00FFFF, 0xFF0F00FF,
                0xFF00F0FF, 0xFFF000FF, 0xFFF0F0FF,
            ][random_range(0..10)];
            return old_params;
        })
        .update(ConstModifier {
            attribute: Attribute::Acceleration,
            value: Value::Constant((0.0, 45.0)),
        })
        .update(LinearDragModifier {
            value: Value::Constant(4.0),
        })
        .build(render_context);

        system.add_effect(firework);
        return State { system };
    }

    fn fixed_step_update(
        &mut self,
        input: &framework::input::InputState,
        renderer_context: &mut framework::render_context::RenderContext,
        dt: f32,
    ) {
        return;
    }

    fn interpolate(
        &mut self,
        alpha: f32,
        renderer_context: &mut framework::render_context::RenderContext,
    ) {
        return;
    }

    fn render(
        &mut self,
        render_context: &mut framework::render_context::RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        time_delta: f32,
    ) {
        let effect = self.system.effects.get_mut("fireworks").unwrap();
        effect.move_origin([
            random_range(200..render_context.surface_size.0 - 200) as f32,
            random_range(200..render_context.surface_size.1 - 200) as f32,
        ]);
        self.system.update(render_context, encoder, time_delta);
        self.system.render(render_context, encoder, surface);
    }
}

fn main() {
    run_app::<State>(Config {
        window_size: (1200, 900),
        title: "test",
        gpu_features: wgpu::Features::default(),
        gpu_limits: wgpu::Limits::default(),
    });
}
