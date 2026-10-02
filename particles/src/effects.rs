use std::default;

use framework::{
    render_context::RenderContext,
    resources::{Binding, StoredBindGroup},
};

use crate::{
    buffers::Buffers,
    modifier::{Attribute, Modifier, ModifierStage},
    pipelines::EffectPipelines,
};

#[derive(default::Default)]
pub enum EmissionMode {
    #[default]
    Constant,
    Burst(u32),
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct Locals {
    vertex_count: u32,
    instance_count: u32,
    _pad: [u32; 2],
    origin: [f32; 2],
    to_emit: u32,
    params: [u32; 4],
    dead_indices_count: i32,
}

impl Locals {
    pub fn dump_writeables(&self) -> &[u8] {
        return &bytemuck::bytes_of(self)[..std::mem::offset_of!(Locals, dead_indices_count)];
    }
}

pub struct Effect {
    pub name: &'static str,
    emission_mode: EmissionMode,
    spawn_interval: f32,
    timer: f32,
    buffers: Buffers,
    pipelines: EffectPipelines,
    locals: Locals,
    render_bind_group: StoredBindGroup,
    param_update_fn: fn([u32; 4]) -> [u32; 4],
}

impl Effect {
    pub fn new(
        render_context: &mut RenderContext,
        position: [f32; 2],
        name: &'static str,
        max_particles: u32,
        emission_mode: EmissionMode,
        spawn_interval: f32,
        emit_shader_code: String,
        update_shader_code: String,
        render_shader_code: String,
        params: [u32; 4],
        param_update_fn: Option<fn([u32; 4]) -> [u32; 4]>,
    ) -> Effect {
        let locals = Locals {
            dead_indices_count: max_particles as i32,
            origin: position,
            to_emit: 0,
            vertex_count: 6,
            instance_count: 0,
            _pad: [0; 2],
            params,
        };
        let buffers = Buffers::new(render_context, name, max_particles, &locals);
        Effect::create_update_bind_group(name, &buffers, render_context);
        let render_bind_group = Effect::create_render_bind_group(name, &buffers, render_context);
        let param_fn = param_update_fn.unwrap_or(|params| params);
        return Effect {
            name,
            emission_mode,
            spawn_interval,
            timer: 0.0,
            buffers,
            pipelines: EffectPipelines::new(
                render_context,
                name,
                max_particles,
                emit_shader_code,
                update_shader_code,
                render_shader_code,
            ),
            locals,
            render_bind_group,
            param_update_fn: param_fn,
        };
    }

    pub fn move_origin(&mut self, new_origin: [f32; 2]) {
        self.locals.origin = new_origin;
    }

    pub fn update(
        &mut self,
        pass: &mut wgpu::ComputePass,
        render_context: &mut RenderContext,
        time_delta: f32,
    ) {
        self.timer += time_delta;
        let to_emit = match self.emission_mode {
            EmissionMode::Constant => {
                let to_emit = self.timer / self.spawn_interval;
                self.timer = self.timer % self.spawn_interval;
                to_emit
            }
            EmissionMode::Burst(ammount) => {
                if self.timer > self.spawn_interval {
                    self.timer %= self.spawn_interval;
                    ammount as f32
                } else {
                    0.0
                }
            }
        };
        self.locals.params = (self.param_update_fn)(self.locals.params);
        self.locals.to_emit = to_emit as u32;
        render_context
            .gpu_resources
            .write_buffer(&self.buffers.locals, self.locals.dump_writeables());

        self.pipelines.emit_pipeline.work_groups = ((to_emit as u32 + 15) / 16, 1, 1);
        self.pipelines
            .emit_pipeline
            .compute(pass, &mut render_context.gpu_resources);
        self.pipelines
            .update_pipeline
            .compute(pass, &mut render_context.gpu_resources);
    }

    pub fn render(&self, render_context: &RenderContext, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.pipelines.render_pipeline.pipeline);
        render_pass.set_bind_group(1, &self.render_bind_group.bind_group, &[]);
        let indirect_buffer = render_context
            .gpu_resources
            .get_buffer(&self.buffers.locals);
        render_pass.draw_indirect(indirect_buffer, 0);
    }

    pub fn create_render_bind_group(
        name: &'static str,
        buffers: &Buffers,
        render_context: &mut RenderContext,
    ) -> StoredBindGroup {
        let bindings = vec![
            Binding::Buffer {
                binding: 0,
                handle: buffers.particle_buffer.clone(),
                buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
            },
            Binding::Buffer {
                binding: 1,
                handle: buffers.dead_indices.clone(),
                buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
            },
            Binding::Buffer {
                binding: 2,
                handle: buffers.alive_indices.clone(),
                buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
            },
        ];
        let layout_handle = render_context
            .gpu_resources
            .create_bind_group_layout(&bindings, "render_layout");
        let build_bind_group =
            render_context
                .gpu_resources
                .build_bind_group(name, layout_handle, bindings);
        render_context
            .gpu_resources
            .bind_groups
            .insert("render_layout", build_bind_group.clone());
        return build_bind_group;
    }

    pub fn create_update_bind_group(
        name: &'static str,
        buffers: &Buffers,
        render_context: &mut RenderContext,
    ) {
        render_context.gpu_resources.create_bind_group(
            name,
            vec![
                Binding::Buffer {
                    binding: 0,
                    handle: buffers.locals.clone(),
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: false },
                },
                Binding::Buffer {
                    binding: 1,
                    handle: buffers.particle_buffer.clone(),
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: false },
                },
                Binding::Buffer {
                    binding: 2,
                    handle: buffers.dead_indices.clone(),
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: false },
                },
                Binding::Buffer {
                    binding: 3,
                    handle: buffers.alive_indices.clone(),
                    buffer_type: wgpu::BufferBindingType::Storage { read_only: false },
                },
            ],
        );
    }
}

pub struct EffectBuilder {
    pub(crate) max_particles: u32,
    name: &'static str,
    init_modifiers: Vec<Box<dyn Modifier>>,
    update_modifiers: Vec<Box<dyn Modifier>>,
    render_modifiers: Vec<Box<dyn Modifier>>,
    emission_mode: EmissionMode,
    spawn_interval: f32,
    position: [f32; 2],
    particle_size: u32,
    params: [u32; 4],
    param_update_fn: Option<fn([u32; 4]) -> [u32; 4]>,
    needs_atlas: bool,
}

impl EffectBuilder {
    pub fn new(name: &'static str, position: [f32; 2], max_particles: u32) -> EffectBuilder {
        return EffectBuilder {
            init_modifiers: Vec::new(),
            update_modifiers: Vec::new(),
            render_modifiers: Vec::new(),
            emission_mode: EmissionMode::default(),
            max_particles,
            spawn_interval: 1.0,
            name,
            position,
            particle_size: 1,
            params: [0; 4],
            param_update_fn: None,
            needs_atlas: false,
        };
    }

    pub fn init<T>(mut self, modifier: T) -> Self
    where
        T: Modifier + 'static,
    {
        assert!(modifier.stage().contains(&ModifierStage::Init));
        let boxed = Box::new(modifier);
        self.init_modifiers.push(boxed);
        return self;
    }

    pub fn update<T>(mut self, modifier: T) -> Self
    where
        T: Modifier + 'static,
    {
        assert!(modifier.stage().contains(&ModifierStage::Update));
        let boxed = Box::new(modifier);
        self.update_modifiers.push(boxed);
        return self;
    }

    pub fn render<T>(mut self, modifier: T) -> Self
    where
        T: Modifier + 'static,
    {
        assert!(modifier.stage().contains(&ModifierStage::Render));
        if matches!(modifier.attribute(), Attribute::SpriteIdx) {
            self.needs_atlas = true;
        };
        let boxed = Box::new(modifier);
        self.render_modifiers.push(boxed);
        return self;
    }

    pub fn emission_mode(mut self, mode: EmissionMode) -> Self {
        self.emission_mode = mode;
        return self;
    }

    pub fn spawn_interval(mut self, interval: f32) -> Self {
        self.spawn_interval = interval;
        return self;
    }

    pub fn particle_size(mut self, particle_size: u32) -> Self {
        self.particle_size = particle_size;
        return self;
    }

    pub fn set_param_slot(mut self, slot: u32, value: u32) -> Self {
        self.params[slot as usize] = value;
        return self;
    }

    pub fn param_update_fn(mut self, function: fn([u32; 4]) -> [u32; 4]) -> Self {
        self.param_update_fn = Some(function);
        return self;
    }

    pub fn build_shader_str(&mut self, stage: ModifierStage) -> String {
        let modifiers = match stage {
            ModifierStage::Init => &mut self.init_modifiers,
            ModifierStage::Update => &mut self.update_modifiers,
            ModifierStage::Render => &mut self.render_modifiers,
        };
        modifiers.sort_by_key(|modifier| modifier.attribute());
        let code = modifiers
            .iter()
            .map(|modifier| modifier.wgsl())
            .collect::<Vec<String>>()
            .concat();
        return code;
    }

    pub fn build(mut self, render_context: &mut RenderContext) -> Effect {
        let emit_code = include_str!("./shaders/emit_particle_template.wgsl")
            .replace("{{ INIT }}", &self.build_shader_str(ModifierStage::Init));

        let update_code = include_str!("./shaders/update_particles_template.wgsl").replace(
            "{{ UPDATE }}",
            &self.build_shader_str(ModifierStage::Update),
        );

        let atlas_binds = if self.needs_atlas {
            "
            @group(0) @binding(1)
            var tex: texture_2d<f32>;
            @group(0) @binding(2)
            var tex_sampler: sampler;
            "
        } else {
            ""
        };
        let render_code = include_str!("./shaders/render_particles_template.wgsl")
            .replace(
                "{{ RENDER }}",
                &self.build_shader_str(ModifierStage::Render),
            )
            .replace("{{ PARTICLE_SIZE }}", &self.particle_size.to_string())
            .replace("{{ ATLAS_BINDS }}", atlas_binds);

        return Effect::new(
            render_context,
            self.position,
            self.name,
            self.max_particles,
            self.emission_mode,
            self.spawn_interval,
            emit_code,
            update_code,
            render_code,
            self.params,
            self.param_update_fn,
        );
    }
}
