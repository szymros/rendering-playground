pub mod framework;
pub mod input;
pub mod instance;
pub mod material;
pub mod mesh;
pub mod pipelines;
pub mod render_context;
pub mod resources;
mod store;
pub mod texture_array;
pub mod vertex;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::Window,
};

use crate::framework::{App, Framework};

pub struct Config {
    pub window_size: (u32, u32),
    pub title: &'static str,
    pub gpu_features: wgpu::Features,
    pub gpu_limits: wgpu::Limits

}

struct Runner<A: App> {
    framework: Option<Framework<A>>,
    config: Config,
}

impl<A: App> Runner<A> {
    pub fn new(config: Config) -> Runner<A> {
        return Runner {
            framework: None,
            config,
        };
    }
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(PhysicalSize::new(
                            self.config.window_size.0,
                            self.config.window_size.1,
                        ))
                        .with_title(self.config.title)
                        .with_visible(true),
                )
                .unwrap(),
        );
        // window.set_cursor_visible(false);re
        // window
        //     .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        //     .unwrap();
        let state = pollster::block_on(Framework::new(window.clone(), &self.config));
        self.framework = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let framework = self.framework.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                framework.update();
                framework.window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => {
                framework.input_state.key_released(key_code);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                framework.input_state.key_pressed(key_code);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                framework.input_state.mouse_pressed(button);
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button,
                ..
            } => {
                framework.input_state.mouse_released(button);
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let state = self.framework.as_mut().unwrap();
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                state.input_state.mouse_moved(delta);
            }
            _ => (),
        }
    }
}

pub fn run_app<A: App>(config: Config) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app: Runner<A> = Runner::new(config);
    let _ = event_loop.run_app(&mut app);
}
