use std::sync::Arc;

use winit::window::Window;

use crate::resources::GpuResources;

pub struct RenderContext {
    pub window: Arc<Window>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
    pub surface_size: (u32, u32),
    pub gpu_resources: GpuResources,
}

impl RenderContext {
    pub async fn new(
        window: Arc<Window>,
        features: wgpu::Features,
        limit: wgpu::Limits,
    ) -> RenderContext {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::wgt::DeviceDescriptor {
                required_features: features,
                required_limits: limit,
                ..Default::default() 
            })
            .await
            .unwrap();
        let device_arc = Arc::new(device);
        let queue_arc = Arc::new(queue);
        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let gpu_resources = GpuResources::new(device_arc.clone(), queue_arc.clone());
        let window_size = window.inner_size();
        let context = RenderContext {
            window,
            device: device_arc,
            queue: queue_arc,
            surface,
            surface_format,
            gpu_resources,
            surface_size: (window_size.width, window_size.height),
        };
        context.configure_surface();
        return context;
    }

    pub fn configure_surface(&self) {
        let size = self.window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
        };
        self.surface.configure(&self.device, &surface_config);
    }
}
