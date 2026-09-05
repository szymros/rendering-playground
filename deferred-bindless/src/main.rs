mod camera;
mod light;
mod model;
mod scene;
use framework::{
    Config,
    framework::App,
    input::InputState,
    instance::{Instance, InstanceBatch, InstanceRegistry},
    material::MaterialRegistry,
    mesh::MeshRegistry,
    pipelines::{RenderPipeline, RenderPipelineBuilder},
    render_context::RenderContext,
    resources::{ATTACHMENT_TEXTURE_USAGES, Binding, GpuResources, TextureResource},
    run_app,
    texture_array::TextureArray,
};
use glam::{Quat, Vec3};
use rand::distr::{Distribution, Uniform};
use wgpu::{BufferBindingType, RenderPassColorAttachment};

use crate::{
    camera::{Camera, CameraController},
    light::LightRegistry,
    model::load_gltf,
};

struct GBuffer {
    albedo: TextureResource,
    frag_pos: TextureResource,
    depth: TextureResource,
    normals: TextureResource,
    material: TextureResource,
}

impl GBuffer {
    pub fn new(gpu_resources: &mut GpuResources, texture_size: (u32, u32)) -> GBuffer {
        let albedo = gpu_resources.create_texture(
            texture_size,
            wgpu::TextureFormat::Rgba8Unorm,
            ATTACHMENT_TEXTURE_USAGES,
        );
        let depth = gpu_resources.create_texture(
            texture_size,
            wgpu::TextureFormat::Depth32Float,
            ATTACHMENT_TEXTURE_USAGES,
        );
        let normals = gpu_resources.create_texture(
            texture_size,
            wgpu::TextureFormat::Rgba16Float,
            ATTACHMENT_TEXTURE_USAGES,
        );
        let frag_pos = gpu_resources.create_texture(
            texture_size,
            wgpu::TextureFormat::Rgba16Float,
            ATTACHMENT_TEXTURE_USAGES,
        );
        let material = gpu_resources.create_texture(
            texture_size,
            wgpu::TextureFormat::Rgba8Unorm,
            ATTACHMENT_TEXTURE_USAGES,
        );
        return GBuffer {
            albedo,
            depth,
            normals,
            frag_pos,
            material,
        };
    }

    pub fn create_bind_group(&self, gpu_resources: &mut GpuResources) {
        gpu_resources.create_bind_group(
            "gbuffer",
            vec![
                Binding::Texture {
                    binding: 0,
                    handle: self.albedo.texture_view_handle.clone(),
                    format: self.albedo.format,
                    bind_type: framework::resources::TextureBindingType::Texture,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                Binding::Texture {
                    binding: 1,
                    handle: self.normals.texture_view_handle.clone(),
                    format: self.normals.format,
                    bind_type: framework::resources::TextureBindingType::Texture,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                Binding::Texture {
                    binding: 2,
                    handle: self.frag_pos.texture_view_handle.clone(),
                    format: self.frag_pos.format,
                    bind_type: framework::resources::TextureBindingType::Texture,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                Binding::Texture {
                    binding: 3,
                    handle: self.material.texture_view_handle.clone(),
                    format: self.material.format,
                    bind_type: framework::resources::TextureBindingType::Texture,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
            ],
        );
    }
}

struct State {
    gbuffer_pipeline: RenderPipeline,
    light_volume_pipeline: RenderPipeline,
    ambient_pipeline: RenderPipeline,
    draw_lights_pipeline: RenderPipeline,
    hdr_blit_pipeline: RenderPipeline,
    camera: Camera,
    camera_controller: CameraController,
    mesh_registry: MeshRegistry,
    instance_registry: InstanceRegistry,
    material_registry: MaterialRegistry,
    light_registry: LightRegistry,
    instances: Vec<InstanceBatch>,
    gbuffer: GBuffer,
    intermediate_tex: TextureResource,
}

impl App for State {
    fn init(render_context: &mut RenderContext) -> Self {
        let model_path = std::env::args().nth(1).expect("no gltf path given");
        let gbuffer = GBuffer::new(
            &mut render_context.gpu_resources,
            render_context.surface_size,
        );
        let intermediate_tex = render_context.gpu_resources.create_texture(
            render_context.surface_size,
            wgpu::TextureFormat::Rgba16Float,
            ATTACHMENT_TEXTURE_USAGES,
        );
        let mut textures = TextureArray::new(&mut render_context.gpu_resources);
        let mut mesh_registry = MeshRegistry::new(&mut render_context.gpu_resources);
        let mut instance_registry = InstanceRegistry::new(&mut render_context.gpu_resources);
        let mut material_registry = MaterialRegistry::new(&mut render_context.gpu_resources);
        let mut light_registry =
            LightRegistry::new(&mut render_context.gpu_resources, &mut mesh_registry);
        gbuffer.create_bind_group(&mut render_context.gpu_resources);
        let (meshes, materials) = load_gltf(
            &model_path,
            &mut render_context.gpu_resources,
            &mut mesh_registry,
            &mut textures,
        );
        material_registry.materials.extend(materials);
        material_registry.flush(&mut render_context.gpu_resources, &render_context.queue);
        let mut transforms: Vec<Instance> = Vec::new();
        let pos_range = Uniform::new(0.0, 2.0).unwrap();
        let color_range = Uniform::new(0.2, 0.8).unwrap();
        let mut rng = rand::rng();
        for i in 0..8 {
            for j in 0..8 {
                transforms.push(Instance::from_translation_rotation_scale(
                    Vec3::new(i as f32 * 6.0, 0.0, j as f32 * 6.0),
                    Quat::from_rotation_x(0.0),
                    Vec3::splat(1.0),
                ));
                light_registry.add_light(
                    Vec3::new(
                        3.0 + i as f32 * 6.0 + pos_range.sample(&mut rng),
                        0.0 + pos_range.sample(&mut rng),
                        3.0 + j as f32 * 6.0 + pos_range.sample(&mut rng),
                    ),
                    Vec3::new(
                        color_range.sample(&mut rng),
                        color_range.sample(&mut rng),
                        color_range.sample(&mut rng),
                    ),
                    6.0,
                    10.0,
                );
            }
        }
        light_registry.flush(&mut render_context.gpu_resources, &mut instance_registry);
        render_context.gpu_resources.create_bind_group(
            "lights_bg",
            vec![Binding::Buffer {
                binding: 0,
                handle: light_registry.buffer_handle.clone(),
                buffer_type: wgpu::BufferBindingType::Storage { read_only: true },
            }],
        );
        let instances: Vec<InstanceBatch> = meshes
            .iter()
            .map(|mesh| instance_registry.build_batch_with_material(&transforms, *mesh))
            .collect();
        instance_registry.flush(&render_context.gpu_resources);
        mesh_registry.flush(&render_context.gpu_resources);

        render_context.gpu_resources.create_bind_group(
            "materials",
            vec![Binding::Buffer {
                binding: 0,
                handle: material_registry.buffer_handle.clone(),
                buffer_type: BufferBindingType::Storage { read_only: true },
            }],
        );
        render_context
            .gpu_resources
            .create_bind_group("textures", textures.binding(0));
        let camera = Camera::new(
            Vec3::new(1.5, 0.0, -5.0),
            &mut render_context.gpu_resources,
            render_context.surface_size,
        );
        camera.update_buffer(
            camera.uniform,
            &render_context.queue,
            &render_context.gpu_resources,
        );
        render_context.gpu_resources.create_bind_group(
            "camera",
            vec![Binding::Buffer {
                binding: 0,
                handle: camera.buffer_handle.clone(),
                buffer_type: wgpu::BufferBindingType::Uniform,
            }],
        );
        let gbuffer_pipeline =
            RenderPipelineBuilder::new("gbuffer", wgpu::include_wgsl!("./shaders/gbuffer.wgsl"))
                .bind_groups(&["camera", "textures", "materials"])
                .render_targets(&[
                    Some(gbuffer.albedo.format.into()),
                    Some(gbuffer.normals.format.into()),
                    Some(gbuffer.frag_pos.format.into()),
                    Some(gbuffer.material.format.into()),
                ])
                .vertex_buffers(
                    mesh_registry.vertices.handle.clone(),
                    mesh_registry.indices.handle.clone(),
                    instance_registry.instance_buffer.handle.clone(),
                )
                .face_culling(wgpu::Face::Back)
                .depth_test(gbuffer.depth.format, wgpu::CompareFunction::Less, true)
                .build(&render_context);

        let light_volume_pipeline = RenderPipelineBuilder::new(
            "light_volume",
            wgpu::include_wgsl!("./shaders/light_pass.wgsl"),
        )
        .bind_groups(&["camera", "lights_bg", "gbuffer"])
        .render_targets(&[Some(wgpu::ColorTargetState {
            format: intermediate_tex.format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        })])
        .vertex_buffers(
            mesh_registry.vertices.handle.clone(),
            mesh_registry.indices.handle.clone(),
            instance_registry.instance_buffer.handle.clone(),
        )
        .depth_test(
            gbuffer.depth.format,
            wgpu::CompareFunction::GreaterEqual,
            false,
        )
        .face_culling(wgpu::Face::Front)
        .build(&render_context);

        let ambient_pipeline =
            RenderPipelineBuilder::new("ambient", wgpu::include_wgsl!("./shaders/ambient.wgsl"))
                .bind_groups(&["gbuffer"])
                .render_targets(&[Some(intermediate_tex.format.into())])
                .build(&render_context);

        render_context.gpu_resources.create_bind_group(
            "intermediate_tex",
            vec![Binding::Texture {
                binding: 0,
                handle: intermediate_tex.texture_view_handle.clone(),
                format: intermediate_tex.format,
                bind_type: framework::resources::TextureBindingType::Texture,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            }],
        );
        let hdr_blit_pipeline =
            RenderPipelineBuilder::new("hdr_blit", wgpu::include_wgsl!("./shaders/hdr_blit.wgsl"))
                .bind_groups(&["intermediate_tex"])
                .render_targets(&[Some(render_context.surface_format.into())])
                .build(&render_context);

        let draw_lights_pipeline = RenderPipelineBuilder::new(
            "draw_light",
            wgpu::include_wgsl!("./shaders/draw_light.wgsl"),
        )
        .bind_groups(&["camera", "lights_bg"])
        .render_targets(&[Some(intermediate_tex.format.into())])
        .vertex_buffers(
            mesh_registry.vertices.handle.clone(),
            mesh_registry.indices.handle.clone(),
            instance_registry.instance_buffer.handle.clone(),
        )
        .depth_test(gbuffer.depth.format, wgpu::CompareFunction::Less, false)
        .build(&render_context);

        let camera_controller = CameraController::new(camera.clone());
        return State {
            gbuffer_pipeline,
            mesh_registry,
            camera,
            camera_controller,
            gbuffer,
            ambient_pipeline,
            light_volume_pipeline,
            instance_registry,
            instances,
            material_registry,
            light_registry,
            draw_lights_pipeline,
            intermediate_tex,
            hdr_blit_pipeline,
        };
    }

    fn fixed_step_update(
        &mut self,
        input_state: &InputState,
        render_context: &mut RenderContext,
        dt: f32,
    ) {
        self.camera_controller
            .update_camera(&mut self.camera, &input_state, dt);
        self.camera.update_buffer(
            self.camera.uniform,
            &render_context.queue,
            &render_context.gpu_resources,
        );
    }

    fn interpolate(&mut self, alpha: f32, render_context: &mut RenderContext) {
        let interpolated_camera = self.camera_controller.interpolate(&self.camera, alpha);
        self.camera.update_buffer(
            interpolated_camera.uniform,
            &render_context.queue,
            &render_context.gpu_resources,
        );
        return;
    }

    fn render(
        &mut self,
        render_context: &mut RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
    ) {
        let depth_view = render_context
            .gpu_resources
            .get_texture_view(&self.gbuffer.depth.texture_view_handle);
        let albedo_view = render_context
            .gpu_resources
            .get_texture_view(&self.gbuffer.albedo.texture_view_handle);
        let normals_view = render_context
            .gpu_resources
            .get_texture_view(&self.gbuffer.normals.texture_view_handle);
        let farg_pos_view = render_context
            .gpu_resources
            .get_texture_view(&self.gbuffer.frag_pos.texture_view_handle);
        let material_view = render_context
            .gpu_resources
            .get_texture_view(&self.gbuffer.material.texture_view_handle);
        let intermediate_view = render_context
            .gpu_resources
            .get_texture_view(&self.intermediate_tex.texture_view_handle);
        let gbuffer_color_attachments = vec![
            Some(wgpu::RenderPassColorAttachment {
                view: albedo_view,
                depth_slice: None,
                resolve_target: None,
                ops: Default::default(),
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: normals_view,
                depth_slice: None,
                resolve_target: None,
                ops: Default::default(),
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: farg_pos_view,
                depth_slice: None,
                resolve_target: None,
                ops: Default::default(),
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: material_view,
                depth_slice: None,
                resolve_target: None,
                ops: Default::default(),
            }),
        ];
        self.gbuffer_pass(
            &render_context,
            encoder,
            &gbuffer_color_attachments,
            &depth_view,
        );
        self.ambient_pass(render_context, encoder, intermediate_view);
        self.light_volume_pass(render_context, encoder, intermediate_view, depth_view);
        self.draw_light_pass(render_context, encoder, intermediate_view, depth_view);
        self.hdr_blit_pass(render_context, encoder, surface);
    }
}

impl State {
    fn gbuffer_pass(
        &self,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        color_attachments: &[Option<RenderPassColorAttachment>],
        depth_buffer_view: &wgpu::TextureView,
    ) {
        let mut geometry_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("gbuffer"),
            color_attachments: color_attachments,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_buffer_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        for instance in self.instances.iter() {
            let mesh = instance.mesh;
            let vertex_range = mesh.vertex_range.0..mesh.vertex_range.1;
            let index_range = mesh.index_range.0..mesh.index_range.1;
            let instance_range = instance.instance_range.0..instance.instance_range.1;
            self.gbuffer_pipeline.draw_indexed(
                &mut geometry_pass,
                &render_context.gpu_resources,
                vertex_range,
                index_range,
                instance_range,
            );
        }
    }

    fn ambient_pass(
        &self,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        intermediate_tex_view: &wgpu::TextureView,
    ) {
        let mut ambient_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ambient"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &intermediate_tex_view,
                depth_slice: None,
                resolve_target: None,
                ops: Default::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        self.ambient_pipeline
            .draw(&mut ambient_pass, &render_context.gpu_resources, 0..3, 0..1);
    }

    fn light_volume_pass(
        &self,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        intermediate_tex_view: &wgpu::TextureView,
        depth_tex_view: &wgpu::TextureView,
    ) {
        let mut light_volume_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("light_volume_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: intermediate_tex_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_tex_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let mesh = self.light_registry.instance_batch.mesh;
        let vertex_range = mesh.vertex_range.0..mesh.vertex_range.1;
        let index_range = mesh.index_range.0..mesh.index_range.1;
        self.light_volume_pipeline.draw_indexed(
            &mut light_volume_pass,
            &render_context.gpu_resources,
            vertex_range.clone(),
            index_range.clone(),
            self.light_registry.instance_batch.instance_range.0
                ..self.light_registry.instance_batch.instance_range.1,
        );
    }

    fn draw_light_pass(
        &self,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        intermediate_tex_view: &wgpu::TextureView,
        depth_tex_view: &wgpu::TextureView,
    ) {
        let mut draw_light_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("draw_light_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: intermediate_tex_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_tex_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let instance = self.light_registry.visualization_instance_batch;
        let mesh = instance.mesh;
        let vertex_range = mesh.vertex_range.0..mesh.vertex_range.1;
        let index_range = mesh.index_range.0..mesh.index_range.1;
        self.draw_lights_pipeline.draw_indexed(
            &mut draw_light_pass,
            &render_context.gpu_resources,
            vertex_range,
            index_range,
            self.light_registry
                .visualization_instance_batch
                .instance_range
                .0
                ..self
                    .light_registry
                    .visualization_instance_batch
                    .instance_range
                    .1,
        );
        drop(draw_light_pass);
    }

    fn hdr_blit_pass(
        &self,
        render_context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
    ) {
        let mut hdr_blit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("hdr_blit"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                depth_slice: None,
                resolve_target: None,
                ops: Default::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        self.hdr_blit_pipeline.draw(
            &mut hdr_blit_pass,
            &render_context.gpu_resources,
            0..3,
            0..1,
        );
    }
}

fn main() {
    run_app::<State>(Config {
        window_size: (1600, 900),
        title: "test",
        gpu_features: wgpu::Features {
            features_wgpu: wgpu::FeaturesWGPU::TEXTURE_BINDING_ARRAY
                | wgpu::FeaturesWGPU::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,

            ..Default::default()
        },
        gpu_limits: wgpu::Limits {
            max_storage_buffers_per_shader_stage: 16,
            max_binding_array_elements_per_shader_stage: 64,
            ..Default::default()
        },
    });
}
