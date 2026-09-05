use std::ops::Range;

use wgpu::{Buffer, VertexBufferLayout};

use crate::instance::Instance;
use crate::render_context::RenderContext;
use crate::resources::{BufferHandle, GpuResources, LayoutHandle};
use crate::vertex::Vertex;

pub struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,
    bind_groups: Vec<&'static str>,
    pub work_groups: (u32, u32, u32),
}

impl ComputePipeline {
    pub fn new(
        render_context: &RenderContext,
        shader_descriptor: wgpu::ShaderModuleDescriptor,
        label: &str,
        bind_groups: &[&'static str],
        work_groups: (u32, u32, u32),
    ) -> ComputePipeline {
        let shader = render_context
            .device
            .create_shader_module(shader_descriptor);
        let layout_handles: Vec<LayoutHandle> = bind_groups
            .iter()
            .map(|handle| {
                render_context
                    .gpu_resources
                    .bind_groups
                    .get(handle)
                    .expect(&format!("missing bind group {}", handle))
                    .layout_handle
                    .clone()
            })
            .collect();
        let layouts: Vec<Option<&wgpu::BindGroupLayout>> = layout_handles
            .iter()
            .map(|handle| {
                Some(
                    render_context
                        .gpu_resources
                        .layouts
                        .get(handle)
                        .expect("hanging layout handle"),
                )
            })
            .collect();
        let pipeline_layout =
            render_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts: &layouts,
                    immediate_size: 0,
                });
        let pipeline =
            render_context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pipeline_layout),
                    module: &shader,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    cache: None,
                });
        return ComputePipeline {
            pipeline,
            bind_groups: bind_groups.to_vec(),
            work_groups,
        };
    }

    pub fn compute(&mut self, compute_pass: &mut wgpu::ComputePass, gpu_resources: &GpuResources) {
        for (idx, bind_group_name) in self.bind_groups.iter().enumerate() {
            let bind_group = gpu_resources
                .bind_groups
                .get(bind_group_name)
                .expect(&format!("Bind group {} missing", bind_group_name));
            compute_pass.set_bind_group(idx as u32, &bind_group.bind_group, &[]);
        }
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.dispatch_workgroups(
            self.work_groups.0,
            self.work_groups.1,
            self.work_groups.2,
        );
    }

    pub fn compute_indirect(
        &mut self,
        compute_pass: &mut wgpu::ComputePass,
        gpu_resources: &GpuResources,
        indirect_handle: BufferHandle,
    ) {
        for (idx, bind_group_name) in self.bind_groups.iter().enumerate() {
            let bind_group = gpu_resources
                .bind_groups
                .get(bind_group_name)
                .expect(&format!("Bind group {} missing", bind_group_name));
            compute_pass.set_bind_group(idx as u32, &bind_group.bind_group, &[]);
        }
        let indirect_buffer = gpu_resources.buffers.get(&indirect_handle).expect("");
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.dispatch_workgroups_indirect(indirect_buffer, 0);
    }
}

pub struct RenderPipelineBuilder<'a> {
    label: &'a str,
    vertex_buffer_handle: Option<BufferHandle>,
    instance_buffer_handle: Option<BufferHandle>,
    index_buffer_handle: Option<BufferHandle>,
    bind_groups: Vec<&'static str>,
    shader_descriptor: wgpu::ShaderModuleDescriptor<'a>,
    face_culling: Option<wgpu::Face>,
    depth_compare: Option<wgpu::CompareFunction>,
    depth_write: Option<bool>,
    depth_format: Option<wgpu::TextureFormat>,
    render_targets: Vec<Option<wgpu::ColorTargetState>>,
    stencil: Option<wgpu::StencilState>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(label: &'a str, shader_descriptor: wgpu::ShaderModuleDescriptor<'a>) -> Self {
        return RenderPipelineBuilder {
            label: label,
            vertex_buffer_handle: None,
            instance_buffer_handle: None,
            index_buffer_handle: None,
            bind_groups: vec![],
            shader_descriptor,
            face_culling: None,
            depth_compare: None,
            depth_write: None,
            depth_format: None,
            render_targets: Vec::new(),
            stencil: None,
        };
    }

    pub fn bind_groups(mut self, bind_groups: &[&'static str]) -> Self {
        self.bind_groups = bind_groups.to_vec();
        return self;
    }

    pub fn vertex_buffers(
        mut self,
        vertex_buffer_handle: BufferHandle,
        index_buffer_handle: BufferHandle,
        instance_buffer_handle: BufferHandle,
    ) -> Self {
        self.vertex_buffer_handle = Some(vertex_buffer_handle);
        self.instance_buffer_handle = Some(instance_buffer_handle);
        self.index_buffer_handle = Some(index_buffer_handle);
        return self;
    }

    pub fn face_culling(mut self, face_culling: wgpu::Face) -> Self {
        self.face_culling = Some(face_culling);
        return self;
    }

    pub fn depth_test(
        mut self,
        depth_format: wgpu::TextureFormat,
        depth_compare: wgpu::CompareFunction,
        depth_write: bool,
    ) -> Self {
        self.depth_compare = Some(depth_compare);
        self.depth_write = Some(depth_write);
        self.depth_format = Some(depth_format);
        return self;
    }

    pub fn render_targets(mut self, render_targets: &[Option<wgpu::ColorTargetState>]) -> Self {
        self.render_targets = render_targets.to_vec();
        return self;
    }

    pub fn stencil(mut self, stencil: wgpu::StencilState) -> Self {
        self.stencil = Some(stencil);
        return self;
    }

    pub fn build(self, render_context: &RenderContext) -> RenderPipeline {
        return RenderPipeline::new(
            &render_context.device,
            self.shader_descriptor,
            self.label,
            &self.bind_groups,
            &render_context.gpu_resources,
            &self.render_targets,
            self.vertex_buffer_handle,
            self.index_buffer_handle,
            self.instance_buffer_handle,
            self.depth_compare,
            self.depth_write,
            self.depth_format,
            self.face_culling,
            self.stencil,
        );
    }
}

pub struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<&'static str>,
    vertex_buffer_handle: Option<BufferHandle>,
    instance_buffer_handle: Option<BufferHandle>,
    index_buffer_handle: Option<BufferHandle>,
}

impl RenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_descriptor: wgpu::ShaderModuleDescriptor,
        label: &str,
        bind_groups: &[&'static str],
        gpu_resources: &GpuResources,
        target_formats: &[Option<wgpu::ColorTargetState>],
        vertex_buffer_handle: Option<BufferHandle>,
        index_buffer_handle: Option<BufferHandle>,
        instance_buffer_handle: Option<BufferHandle>,
        depth_test: Option<wgpu::CompareFunction>,
        depth_write: Option<bool>,
        depth_format: Option<wgpu::TextureFormat>,
        cull_mode: Option<wgpu::Face>,
        opt_stencil: Option<wgpu::StencilState>,
    ) -> RenderPipeline {
        let shader = device.create_shader_module(shader_descriptor);
        let layout_handles: Vec<LayoutHandle> = bind_groups
            .iter()
            .map(|handle| {
                gpu_resources
                    .bind_groups
                    .get(handle)
                    .expect(&format!("missing bind group {}", handle))
                    .layout_handle
                    .clone()
            })
            .collect();
        let layouts: Vec<Option<&wgpu::BindGroupLayout>> = layout_handles
            .iter()
            .map(|handle| {
                Some(
                    gpu_resources
                        .layouts
                        .get(handle)
                        .expect("dangling layout handle"),
                )
            })
            .collect();
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}_layout", label)),
            bind_group_layouts: &layouts,
            immediate_size: 0,
        });
        let depth_stencil = if depth_test.is_some() {
            Some(wgpu::DepthStencilState {
                format: depth_format.unwrap_or(wgpu::TextureFormat::Depth32Float),
                depth_write_enabled: depth_write,
                depth_compare: depth_test,
                stencil: opt_stencil.unwrap_or_default(),
                bias: wgpu::DepthBiasState::default(),
            })
        } else {
            None
        };
        let vertex_buffer_desc: Vec<VertexBufferLayout> = if vertex_buffer_handle.is_some() {
            let mut desc = vec![Vertex::desc()];
            if instance_buffer_handle.is_some() {
                desc.push(Instance::desc());
            }
            desc
        } else {
            vec![]
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &vertex_buffer_desc,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: target_formats,
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: cull_mode,
                ..Default::default()
            },
            depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        return RenderPipeline {
            pipeline,
            bind_groups: bind_groups.to_vec(),
            vertex_buffer_handle,
            index_buffer_handle,
            instance_buffer_handle,
        };
    }

    pub fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass,
        gpu_resources: &GpuResources,
        vertex_range: Range<u32>,
        instances: Range<u32>,
    ) {
        for (idx, bind_group_name) in self.bind_groups.iter().enumerate() {
            let bind_group = gpu_resources
                .bind_groups
                .get(bind_group_name)
                .expect(&format!("Bind group {} missing", bind_group_name));
            render_pass.set_bind_group(idx as u32, &bind_group.bind_group, &[]);
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.draw(vertex_range, instances);
    }

    pub fn draw_indexed(
        &self,
        render_pass: &mut wgpu::RenderPass,
        gpu_resources: &GpuResources,
        vertex_range: Range<u32>,
        index_range: Range<u32>,
        instances: Range<u32>,
    ) {
        for (idx, bind_group_name) in self.bind_groups.iter().enumerate() {
            let bind_group = gpu_resources
                .bind_groups
                .get(bind_group_name)
                .expect(&format!("Bind group {} missing", bind_group_name));
            render_pass.set_bind_group(idx as u32, &bind_group.bind_group, &[]);
        }
        render_pass.set_pipeline(&self.pipeline);
        if let (Some(vertex_buffer_handle), Some(index_buffer_handle)) = (
            self.vertex_buffer_handle.clone(),
            self.index_buffer_handle.clone(),
        ) {
            let vertex_buffer = gpu_resources.get_buffer(&vertex_buffer_handle);
            let index_buffer = gpu_resources.get_buffer(&index_buffer_handle);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        }
        if let Some(instance_buffer_handle) = self.instance_buffer_handle.clone() {
            let instance_buffer = gpu_resources.get_buffer(&instance_buffer_handle);
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        }
        render_pass.draw_indexed(index_range, vertex_range.start as i32, instances);
    }
}
