use glam::{Quat, Vec3};
use framework::{
    instance::{Instance, InstanceBatch, InstanceRegistry},
    mesh::{Mesh, MeshRegistry},
    resources::{BufferHandle, GpuResources, STORAGE_BUFFER_USAGES},
};

use crate::model::load_mesh_from_obj;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    pub color: [f32; 4],
    pub pos: [f32; 4],
    pub radius: f32,
    pub intensity: f32,
    _padding: [u32;2]
}

pub struct LightRegistry {
    pub buffer_handle: BufferHandle,
    pub instance_batch: InstanceBatch,
    pub visualization_instance_batch: InstanceBatch,
    lights: Vec<Light>,
    light_instances: Vec<Instance>,
    light_mesh: Mesh,
    visualization_instances: Vec<Instance>,
}

impl LightRegistry {
    pub fn new(
        gpu_resources: &mut GpuResources,
        mesh_registry: &mut MeshRegistry,
    ) -> LightRegistry {
        let sphere = load_mesh_from_obj("./assets/sphere/sphere.obj", mesh_registry);
        let buffer_handle = gpu_resources.create_buffer(
            std::mem::size_of::<Light>() as u64 * 256,
            "light_buffer",
            STORAGE_BUFFER_USAGES,
        );
        return LightRegistry {
            buffer_handle,
            lights: Vec::new(),
            light_instances: Vec::new(),
            light_mesh: sphere[0],
            instance_batch: Default::default(),
            visualization_instance_batch: Default::default(),
            visualization_instances: Vec::new(),
        };
    }

    pub fn add_light(
        &mut self,
        pos: Vec3,
        color: Vec3,
        radius: f32,
        intensity: f32,
    ) {
        self.lights.push(Light {
            color: color.extend(0.0).to_array(),
            pos: pos.extend(0.0).to_array(),
            intensity,
            radius,
            _padding: [0;2]
        });
        self.light_instances
            .push(Instance::from_translation_rotation_scale_material(
                pos,
                Quat::from_rotation_x(0.0),
                Vec3::splat(radius),
                self.lights.len() as i32 - 1,
            ));
        self.visualization_instances
            .push(Instance::from_translation_rotation_scale_material(
                pos,
                Quat::from_rotation_x(0.0),
                Vec3::splat(0.25),
                self.lights.len() as i32 - 1,
            ));
    }

    pub fn flush(
        &mut self,
        gpu_resources: &mut GpuResources,
        instance_registry: &mut InstanceRegistry,
    ) {
        gpu_resources.write_buffer(&self.buffer_handle, bytemuck::cast_slice(&self.lights));
        // if self.instance_batch.instance_range.1 > 0 {
        //     instance_registry.instance_buffer.free(
        //         self.instance_batch.instance_range.0,
        //         self.instance_batch.instance_range.1 - self.instance_batch.instance_range.0,
        //     );
        // }
        self.instance_batch = instance_registry.build_batch(&self.light_instances, self.light_mesh);

        // if self.visualization_instance_batch.instance_range.1 > 0 {
        //     instance_registry.instance_buffer.free(
        //         self.visualization_instance_batch.instance_range.0,
        //         self.visualization_instance_batch.instance_range.1
        //             - self.visualization_instance_batch.instance_range.0,
        //     );
        // }
        self.visualization_instance_batch =
            instance_registry.build_batch(&self.visualization_instances, self.light_mesh);
    }
}
