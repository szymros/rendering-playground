use crate::{Config, input::InputState, render_context::RenderContext};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);

pub trait App {
    fn init(render_context: &mut RenderContext) -> Self;

    fn fixed_step_update(
        &mut self,
        input: &InputState,
        renderer_context: &mut RenderContext,
        dt: f32,
    );
    fn interpolate(&mut self, alpha: f32, renderer_context: &mut RenderContext);

    fn render(
        &mut self,
        renderer_context: &mut RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
    );
}

pub struct Framework<A: App> {
    pub window: Arc<winit::window::Window>,
    pub input_state: InputState,
    app: A,
    render_context: RenderContext,
    last_updated: Instant,
    time_accum: Duration,
}

impl<A: App> Framework<A> {
    pub async fn new(window: Arc<winit::window::Window>, config: &Config) -> Framework<A> {
        let mut render_context = RenderContext::new(
            window.clone(),
            config.gpu_features.clone(),
            config.gpu_limits.clone(),
        )
        .await;
        let input_state = InputState::new();
        let last_updated = Instant::now();
        let time_accum = Duration::default();
        let app = A::init(&mut render_context);
        return Framework {
            window,
            render_context,
            input_state,
            last_updated,
            time_accum,
            app,
        };
    }

    pub fn fixed_step_update(&mut self) {}

    pub fn interpolate(&mut self, time_alpha: f32) {}

    pub fn update(&mut self) {
        let mut delta = self.last_updated.elapsed();
        self.last_updated = Instant::now();
        if delta > Duration::from_millis(250) {
            delta = Duration::from_millis(250);
        };
        self.time_accum += delta;
        while self.time_accum > FRAME_TIME {
            self.time_accum -= FRAME_TIME;
            self.app.fixed_step_update(
                &self.input_state,
                &mut self.render_context,
                FRAME_TIME.as_secs_f32(),
            );
            self.input_state.clear();
        }
        let time_alpha = f32::clamp(
            self.time_accum.as_secs_f32() / FRAME_TIME.as_secs_f32(),
            0.0,
            1.0,
        );
        self.app.interpolate(time_alpha, &mut self.render_context);
        self.render();
    }
    pub fn render(&mut self) {
        let surface_texture = match self.render_context.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Suboptimal(_) => {
                self.render_context.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Validation => {
                panic!();
            }
        };
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.render_context.surface_format),
                ..Default::default()
            });
        let mut encoder = self
            .render_context
            .device
            .create_command_encoder(&Default::default());
        self.app
            .render(&mut self.render_context, &mut encoder, &texture_view);
        self.render_context.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}
