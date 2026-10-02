use std::{collections::HashMap, time::Instant};

use framework::{
    render_context::RenderContext,
    resources::{
        Binding, BufferHandle, TextureBindingType, TextureResource,
        UNIFORM_BUFFER_USAGES,
    },
};

use crate::effects::Effect;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    proj: [[f32; 4]; 4],
    seed: u32,
    time_delta: f32,
    _pad: [u32; 2],
}

pub struct ParticleSystem {
    pub effects: HashMap<&'static str, Effect>,
    last_updated: Instant,
    globals_buffer: BufferHandle,
    texture_atlas: Option<TextureResource>,
}

impl ParticleSystem {
    pub fn new(
        render_context: &mut RenderContext,
        texture_atlas: Option<TextureResource>,
        seed: u32,
    ) -> ParticleSystem {
        let globals_buffer = render_context.gpu_resources.create_buffer(
            std::mem::size_of::<Globals>() as u64,
            "globals_buffer",
            UNIFORM_BUFFER_USAGES,
        );
        let projection_matrix = glam::camera::lh::proj::directx::orthographic(
            0.0,
            render_context.surface_size.0 as f32,
            render_context.surface_size.1 as f32,
            0.0,
            -1.0,
            1.0,
        );
        render_context.gpu_resources.write_buffer(
            &globals_buffer,
            bytemuck::cast_slice(&projection_matrix.to_cols_array()),
        );
        render_context
            .gpu_resources
            .write_buffer_offset(&globals_buffer, &seed.to_le_bytes(), 64);
        let mut global_bindings = vec![Binding::Buffer {
            binding: 0,
            handle: globals_buffer.clone(),
            buffer_type: wgpu::BufferBindingType::Uniform,
        }];
        if let Some(atlas) = &texture_atlas {
            let sampler = render_context.gpu_resources.create_sampler();
            global_bindings.push(Binding::Texture {
                binding: 1,
                handle: atlas.texture_view_handle.clone(),
                format: atlas.format,
                bind_type: TextureBindingType::Texture,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            });
            global_bindings.push(Binding::Sampler {
                binding: 2,
                handle: sampler,
            });
        }
        render_context
            .gpu_resources
            .create_bind_group("globals", global_bindings);

        return ParticleSystem {
            effects: HashMap::new(),
            last_updated: Instant::now(),
            globals_buffer,
            texture_atlas,
        };
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.insert(effect.name, effect);
    }

    pub fn update(
        &mut self,
        render_context: &mut RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        time_delta: f32,
    ) {
        self.last_updated = Instant::now();
        render_context.gpu_resources.write_buffer_offset(
            &self.globals_buffer,
            &time_delta.to_le_bytes(),
            68,
        );
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("update_and_emit_particles"),
            timestamp_writes: None,
        });
        for effect in self.effects.values_mut() {
            effect.update(&mut pass, render_context, time_delta);
        }
    }

    pub fn render(
        &mut self,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("draw_particles"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let globals_bg = render_context
            .gpu_resources
            .bind_groups
            .get("globals")
            .unwrap();
        pass.set_bind_group(0, &globals_bg.bind_group, &[]);
        for effect in self.effects.values_mut() {
            effect.render(render_context, &mut pass);
        }
    }
}
