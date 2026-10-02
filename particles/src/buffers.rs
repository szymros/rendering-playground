use framework::{
    render_context::RenderContext,
    resources::{BufferHandle, STORAGE_BUFFER_USAGES},
};

use crate::{effects::Locals, particle::Particle};

pub struct Buffers {
    pub particle_buffer: BufferHandle,
    pub dead_indices: BufferHandle,
    pub alive_indices: BufferHandle,
    pub locals: BufferHandle,
}

impl Buffers {
    pub fn new(
        render_context: &mut RenderContext,
        effect_name: &'static str,
        max_particles: u32,
        locals: &Locals,
    ) -> Buffers {
        let particle_buffer = render_context.gpu_resources.create_buffer(
            std::mem::size_of::<Particle>() as u64 * max_particles as u64,
            &format!("{}_particle_buffer", effect_name),
            STORAGE_BUFFER_USAGES,
        );
        let dead_indices = render_context.gpu_resources.create_buffer(
            4 * max_particles as u64,
            &format!("{}_dead_indices", effect_name),
            STORAGE_BUFFER_USAGES,
        );
        render_context.gpu_resources.write_buffer(
            &dead_indices,
            bytemuck::cast_slice(&(0..max_particles).collect::<Vec<u32>>()),
        );
        let alive_indices = render_context.gpu_resources.create_buffer(
            4 * max_particles as u64,
            &format!("{}_alive_indices", effect_name),
            STORAGE_BUFFER_USAGES,
        );
        let locals_buffer = render_context.gpu_resources.create_buffer(
            std::mem::size_of::<Locals>() as u64,
            &format!("{}_locals", effect_name),
            STORAGE_BUFFER_USAGES | wgpu::BufferUsages::INDIRECT,
        );
        render_context
            .gpu_resources
            .write_buffer(&locals_buffer, bytemuck::cast_slice(&[*locals]));
        return Buffers {
            particle_buffer,
            dead_indices,
            alive_indices,
            locals: locals_buffer,
        };
    }
}
