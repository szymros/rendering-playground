use crate::resources::{BufferHandle, GpuResources, STORAGE_BUFFER_USAGES};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Material {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub _padding: [u32; 2],
    pub emissive: [f32; 3],
    pub base_color_texture_idx: i32,
    pub metallic_roughness_texture_idx: i32,
    pub normal_texture_idx: i32,
    pub occlusion_texture_idx: i32,
    pub emissive_texture_idx: i32,
    pub _padding1: u32,
}

pub struct MaterialRegistry {
    pub materials: Vec<Material>,
    pub buffer_handle: BufferHandle,
}

impl MaterialRegistry {
    pub fn new(gpu_resources: &mut GpuResources) -> MaterialRegistry {
        let materials = Vec::new();
        let buffer_handle =
            gpu_resources.create_buffer(512 * 4, "materials_buffer", STORAGE_BUFFER_USAGES);
        return MaterialRegistry {
            materials,
            buffer_handle,
        };
    }

    pub fn flush(&self, gpu_resources: &mut GpuResources, queue: &wgpu::Queue) {
        let buffer = gpu_resources.get_buffer(&self.buffer_handle);
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.materials));
    }
}
