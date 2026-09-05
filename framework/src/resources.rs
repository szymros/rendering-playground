use std::any::type_name;
use std::collections::{HashMap, HashSet};
use std::num::NonZero;
use std::sync::Arc;

use crate::store::{Handle, Slab};

pub const UNIFORM_BUFFER_USAGES: wgpu::BufferUsages =
    wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST);
pub const STORAGE_BUFFER_USAGES: wgpu::BufferUsages =
    wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_DST);
pub const MAX_STORAGE_BUFF_SIZE: u64 = 134217728;

pub const STORAGE_TEXTURE_USAGES: wgpu::TextureUsages = wgpu::TextureUsages::COPY_DST
    .union(wgpu::TextureUsages::TEXTURE_BINDING)
    .union(wgpu::TextureUsages::STORAGE_BINDING);
pub const ATTACHMENT_TEXTURE_USAGES: wgpu::TextureUsages = wgpu::TextureUsages::COPY_DST
    .union(wgpu::TextureUsages::TEXTURE_BINDING)
    .union(wgpu::TextureUsages::RENDER_ATTACHMENT);

pub type BufferHandle = Handle<wgpu::Buffer>;
pub type TextureViewHandle = Handle<wgpu::TextureView>;
pub type TextureHandle = Handle<wgpu::Texture>;
pub type SamplerHandle = Handle<wgpu::Sampler>;
pub type LayoutHandle = Handle<wgpu::BindGroupLayout>;

pub enum TextureBindingType {
    Storage,
    Texture,
}

pub enum Binding {
    Buffer {
        binding: u32,
        handle: BufferHandle,
        buffer_type: wgpu::BufferBindingType,
    },
    Texture {
        binding: u32,
        handle: TextureViewHandle,
        format: wgpu::TextureFormat,
        bind_type: TextureBindingType,
        sample_type: wgpu::TextureSampleType,
    },
    TextureArray {
        binding: u32,
        handles: Vec<TextureViewHandle>,
        format: wgpu::TextureFormat,
        bind_type: TextureBindingType,
        sample_type: wgpu::TextureSampleType,
        count: u32,
    },
    Sampler {
        binding: u32,
        handle: SamplerHandle,
    },
}

pub struct StoredBindGroup {
    pub layout_handle: LayoutHandle,
    pub bindings: Vec<Binding>,
    pub bind_group: wgpu::BindGroup,
}

pub struct TextureResource {
    pub texture_handle: TextureHandle,
    pub texture_view_handle: TextureViewHandle,
    pub format: wgpu::TextureFormat,
}

pub struct GpuResources {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pub buffers: Slab<wgpu::Buffer>,
    pub texture_views: Slab<wgpu::TextureView>,
    pub textures: Slab<wgpu::Texture>,
    pub samplers: Slab<wgpu::Sampler>,
    pub layouts: Slab<wgpu::BindGroupLayout>,
    pub bind_groups: HashMap<&'static str, StoredBindGroup>,
}

impl GpuResources {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> GpuResources {
        return GpuResources {
            buffers: Slab::new(),
            texture_views: Slab::new(),
            textures: Slab::new(),
            samplers: Slab::new(),
            layouts: Slab::new(),
            bind_groups: HashMap::new(),
            device,
            queue,
        };
    }

    pub fn create_buffer(
        &mut self,
        size: u64,
        label: &str,
        usage: wgpu::BufferUsages,
    ) -> BufferHandle {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });
        let handle = self.buffers.insert(buffer);
        return handle;
    }

    pub fn write_buffer(&self, handle: &BufferHandle, data: &[u8]) {
        let buffer = self.buffers.get(handle).expect("dangling buffer handle");
        self.queue.write_buffer(buffer, 0, data);
    }

    pub fn get_buffer(&self, buffer_handle: &BufferHandle) -> &wgpu::Buffer {
        return self
            .buffers
            .get(buffer_handle)
            .expect("dangling buffer handle");
    }

    pub fn create_texture(
        &mut self,
        tex_size: (u32, u32),
        format: wgpu::TextureFormat,
        texture_usages: wgpu::TextureUsages,
    ) -> TextureResource {
        let texture_size = wgpu::Extent3d {
            width: tex_size.0,
            height: tex_size.1,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(""),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: texture_usages,
            view_formats: &[format],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let texture_handle = self.textures.insert(texture);
        let texture_view_handle = self.texture_views.insert(texture_view);
        return TextureResource {
            texture_handle,
            texture_view_handle,
            format,
        };
    }

    pub fn create_texture_from_image(
        &mut self,
        image: &[u8],
        dimensions: (u32, u32),
        format: wgpu::TextureFormat,
        texture_usages: wgpu::TextureUsages,
    ) -> TextureResource {
        let tex_size = dimensions;
        let texture_res = self.create_texture(tex_size, format, texture_usages);
        let texture = self.get_texture(&texture_res.texture_handle);
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(format.block_copy_size(None).unwrap_or(1) as u32 * tex_size.0),
                rows_per_image: Some(tex_size.1),
            },
            wgpu::Extent3d {
                width: tex_size.0,
                height: tex_size.1,
                depth_or_array_layers: 1,
            },
        );
        return texture_res;
    }

    pub fn create_sampler(&mut self) -> SamplerHandle {
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let handle = self.samplers.insert(sampler);
        return handle;
    }

    pub fn get_texture(&self, texture_handle: &TextureHandle) -> &wgpu::Texture {
        return self
            .textures
            .get(texture_handle)
            .expect("dangling texture_handle");
    }

    pub fn get_texture_view(&self, texture_view_handle: &TextureViewHandle) -> &wgpu::TextureView {
        return self
            .texture_views
            .get(texture_view_handle)
            .expect("dangling texture_view handle");
    }

    pub fn get_sampler(&self, sampler_handle: &SamplerHandle) -> &wgpu::Sampler {
        return self
            .samplers
            .get(sampler_handle)
            .expect("dangling sampler handle");
    }

    pub fn create_bind_group_layout(&mut self, entries: &[Binding], name: &str) -> LayoutHandle {
        let layout_entries: Vec<wgpu::BindGroupLayoutEntry> = entries
            .iter()
            .map(|entry| match entry {
                Binding::Buffer {
                    binding,
                    buffer_type: wgpu::BufferBindingType::Uniform,
                    ..
                } => wgpu::BindGroupLayoutEntry {
                    binding: *binding,
                    visibility: wgpu::ShaderStages::COMPUTE
                        | wgpu::ShaderStages::FRAGMENT
                        | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                Binding::Buffer {
                    binding,
                    buffer_type: wgpu::BufferBindingType::Storage { read_only },
                    ..
                } => wgpu::BindGroupLayoutEntry {
                    binding: *binding,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: *read_only,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                Binding::Texture {
                    binding,
                    format,
                    bind_type,
                    sample_type,
                    ..
                } => wgpu::BindGroupLayoutEntry {
                    binding: *binding,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: match bind_type {
                        TextureBindingType::Storage => wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: *format,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        TextureBindingType::Texture => wgpu::BindingType::Texture {
                            sample_type: *sample_type,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                    },
                    count: None,
                },
                Binding::Sampler { binding, .. } => wgpu::BindGroupLayoutEntry {
                    binding: *binding,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                Binding::TextureArray {
                    binding,
                    format,
                    bind_type,
                    sample_type,
                    count,
                    ..
                } => wgpu::BindGroupLayoutEntry {
                    binding: *binding,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: match bind_type {
                        TextureBindingType::Storage => wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: *format,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        TextureBindingType::Texture => wgpu::BindingType::Texture {
                            sample_type: *sample_type,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                    },
                    count: NonZero::new(*count),
                },
            })
            .collect();
        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(name),
                    entries: &layout_entries,
                });
        let handle = self.layouts.insert(bind_group_layout);
        return handle;
    }

    pub fn create_bind_group(&mut self, name: &'static str, bindings: Vec<Binding>) {
        let layout_handle = self.create_bind_group_layout(&bindings, name);
        let stored_bind_group = self.build_bind_group(name, layout_handle, bindings);
        self.bind_groups.insert(name, stored_bind_group);
    }

    pub fn build_bind_group(
        &mut self,
        name: &str,
        layout_handle: LayoutHandle,
        bindings: Vec<Binding>,
    ) -> StoredBindGroup {
        let layout = self
            .layouts
            .get(&layout_handle)
            .expect(&format!("hanging bind layout handle for {}", name));
        let mut texture_array: Vec<&wgpu::TextureView> = Vec::new();
        for binding in bindings.iter() {
            if let Binding::TextureArray { handles, .. } = binding {
                for handle in handles.iter() {
                    let tex_view = self.get_texture_view(handle);
                    texture_array.push(tex_view);
                }
            }
        }
        let bind_group_entires: Vec<wgpu::BindGroupEntry> = bindings
            .iter()
            .map(|(entry)| -> wgpu::BindGroupEntry<'_> {
                match entry {
                    Binding::Buffer {
                        binding, handle, ..
                    } => {
                        let buffer = self
                            .buffers
                            .get(handle)
                            .expect(&format!("dangling buffer handle for {}", name));
                        wgpu::BindGroupEntry {
                            binding: *binding,
                            resource: buffer.as_entire_binding(),
                        }
                    }
                    Binding::Texture {
                        binding, handle, ..
                    } => {
                        let texture = self
                            .texture_views
                            .get(handle)
                            .expect(&format!("dangling texture handle for {}", name));
                        wgpu::BindGroupEntry {
                            binding: *binding,
                            resource: wgpu::BindingResource::TextureView(texture),
                        }
                    }
                    Binding::Sampler { binding, handle } => {
                        let sampler = self
                            .samplers
                            .get(handle)
                            .expect(&format!("dangling texture handle for {}", name));
                        wgpu::BindGroupEntry {
                            binding: *binding,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        }
                    }
                    Binding::TextureArray { binding, .. } => wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: wgpu::BindingResource::TextureViewArray(&texture_array),
                    },
                }
            })
            .collect();

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout,
            entries: &bind_group_entires,
        });
        return StoredBindGroup {
            layout_handle,
            bindings,
            bind_group,
        };
    }
}

pub struct PoolAllocBuffer<T> {
    pub data: Vec<T>,
    free_slots: HashMap<u32, Vec<u32>>,
    current_cap: u32,
    max_cap: u32,
    staged: HashSet<u32>,
    pub handle: BufferHandle,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Clone + Copy + Default> PoolAllocBuffer<T> {
    pub fn new(
        gpu_resources: &mut GpuResources,
        capacity: u32,
        usages: wgpu::BufferUsages,
    ) -> PoolAllocBuffer<T> {
        let handle = gpu_resources.create_buffer(
            capacity as u64,
            &format!("pool_allocated_buffer_{}", type_name::<T>()),
            usages,
        );
        return PoolAllocBuffer {
            data: Vec::new(),
            free_slots: HashMap::new(),
            staged: HashSet::new(),
            handle,
            current_cap: 0,
            max_cap: capacity,
        };
    }

    pub fn alloc(&mut self, space: u32) -> u32 {
        let rounded_space = space.next_power_of_two();
        if let Some(idx) = self
            .free_slots
            .get_mut(&rounded_space)
            .and_then(|v| v.pop())
        {
            self.staged.insert(idx);
            return idx;
        }
        let idx = self.current_cap as u32;
        self.current_cap += rounded_space + 1;
        self.data.resize(self.current_cap as usize, T::default());
        return idx;
    }

    pub fn alloc_and_write(&mut self, data: &[T]) -> u32 {
        let idx = self.alloc(data.len() as u32);
        self.data[idx as usize..idx as usize + data.len()].copy_from_slice(data);
        return idx;
    }

    pub fn ensure_enough_cap() {
        todo!();
    }

    pub fn free(&mut self, idx: u32, space: u32) {
        let bucket = space.next_power_of_two();
        self.free_slots
            .entry(bucket)
            .and_modify(|v| v.push(idx))
            .or_insert(vec![idx]);
    }

    pub fn flush(&mut self, gpu_resources: &GpuResources) {
        if self.current_cap > self.max_cap {
            // throw oom
            return;
        }
        gpu_resources.write_buffer(&self.handle, bytemuck::cast_slice(&self.data));
    }
}
