use crate::{
    resources::{GpuResources, MAX_STORAGE_BUFF_SIZE, PoolAllocBuffer},
    vertex::Vertex,
};

#[derive(Copy, Clone, Default)]
pub struct Mesh {
    pub vertex_range: (u32, u32),
    pub index_range: (u32, u32),
}

pub struct MeshRegistry {
    pub vertices: PoolAllocBuffer<Vertex>,
    pub indices: PoolAllocBuffer<u32>,
}

impl MeshRegistry {
    pub fn new(gpu_rescouces: &mut GpuResources) -> MeshRegistry {
        let vertices: PoolAllocBuffer<Vertex> = PoolAllocBuffer::new(
            gpu_rescouces,
            MAX_STORAGE_BUFF_SIZE as u32,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );
        let indices: PoolAllocBuffer<u32> = PoolAllocBuffer::new(
            gpu_rescouces,
            MAX_STORAGE_BUFF_SIZE as u32,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        );
        return MeshRegistry { vertices, indices };
    }
    pub fn upload(&mut self, vertices: &[Vertex], indices: &[u32]) -> Mesh {
        let vertex_idx = self.vertices.alloc_and_write(vertices);
        let index_idx = self.indices.alloc_and_write(indices);
        return Mesh {
            vertex_range: (vertex_idx, vertex_idx + vertices.len() as u32),
            index_range: (index_idx, index_idx + indices.len() as u32),
        };
    }

    pub fn flush(&mut self, gpu_resources: &GpuResources) {
        self.vertices.flush(gpu_resources);
        self.indices.flush(gpu_resources);
    }
}
