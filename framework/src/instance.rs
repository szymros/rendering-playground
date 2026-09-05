use crate::{
    mesh::Mesh,
    resources::{GpuResources, MAX_STORAGE_BUFF_SIZE, PoolAllocBuffer},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Instance {
    pub transform: [[f32; 4]; 4],
    pub material_id: i32,
}

impl Instance {
    pub fn from_translation_rotation_scale(
        translation: glam::f32::Vec3,
        rotation: glam::f32::Quat,
        scale: glam::f32::Vec3,
    ) -> Instance {
        let transform =
            glam::f32::Mat4::from_scale_rotation_translation(scale, rotation, translation)
                .to_cols_array_2d();
        return Instance {
            transform,
            material_id: -1,
        };
    }

    pub fn from_translation_rotation_scale_material(
        translation: glam::f32::Vec3,
        rotation: glam::f32::Quat,
        scale: glam::f32::Vec3,
        material_id: i32,
    ) -> Instance {
        let transform =
            glam::f32::Mat4::from_scale_rotation_translation(scale, rotation, translation)
                .to_cols_array_2d();
        return Instance {
            transform,
            material_id,
        };
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        return wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32,
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 8,
                },
            ],
        };
    }
}

#[derive(Clone, Copy, Default)]
pub struct InstanceBatch {
    pub mesh: Mesh,
    pub instance_range: (u32, u32),
}

pub struct InstanceRegistry {
    pub instance_buffer: PoolAllocBuffer<Instance>,
}

impl InstanceRegistry {
    pub fn new(gpu_resources: &mut GpuResources) -> InstanceRegistry {
        let instance_buffer = PoolAllocBuffer::new(
            gpu_resources,
            MAX_STORAGE_BUFF_SIZE as u32,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );
        return InstanceRegistry { instance_buffer };
    }

    pub fn build_batch_with_material(
        &mut self,
        instances: &[Instance],
        mesh: (Mesh, u32),
    ) -> InstanceBatch {
        let instances_with_material: Vec<Instance> = instances
            .to_vec()
            .iter_mut()
            .map(|instance| {
                instance.material_id = mesh.1 as i32;
                *instance
            })
            .collect();
        let idx = self
            .instance_buffer
            .alloc_and_write(&instances_with_material);
        let instance_range = (idx, idx + instances_with_material.len() as u32);
        return InstanceBatch {
            mesh: mesh.0,
            instance_range,
        };
    }

    pub fn build_batch(&mut self, instances: &[Instance], mesh: Mesh) -> InstanceBatch {
        let idx = self.instance_buffer.alloc_and_write(&instances);
        let instance_range = (idx, idx + instances.len() as u32);
        return InstanceBatch {
            mesh,
            instance_range,
        };
    }

    pub fn flush(&mut self, gpu_resources: &GpuResources) {
        self.instance_buffer.flush(gpu_resources);
    }
}
