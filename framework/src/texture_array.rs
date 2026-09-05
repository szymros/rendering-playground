use image::RgbaImage;

use crate::resources::{
    Binding, GpuResources, STORAGE_TEXTURE_USAGES, SamplerHandle, TextureBindingType,
    TextureViewHandle,
};

pub struct TextureArray {
    pub texture_views: Vec<TextureViewHandle>,
    pub sampler: SamplerHandle,
}

impl TextureArray {
    pub fn new(gpu_resources: &mut GpuResources) -> TextureArray {
        let sampler = gpu_resources.create_sampler();
        return TextureArray {
            texture_views: Vec::new(),
            sampler,
        };
    }

    pub fn add_texture(
        &mut self,
        image: &[u8],
        dimensions: (u32, u32),
        format: wgpu::TextureFormat,
        gpu_resources: &mut GpuResources,
    ) -> u32 {
        let texture_resource = gpu_resources.create_texture_from_image(
            image,
            dimensions,
            format,
            STORAGE_TEXTURE_USAGES,
        );
        let texture_idx = self.texture_views.len();
        self.texture_views
            .push(texture_resource.texture_view_handle);
        return texture_idx as u32;
    }

    pub fn binding(&self, offset: u32) -> Vec<Binding> {
        return vec![
            Binding::TextureArray {
                binding: offset,
                handles: self.texture_views.clone(),
                format: wgpu::TextureFormat::Rgba8Unorm,
                bind_type: TextureBindingType::Texture,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                count: self.texture_views.len() as u32,
            },
            Binding::Sampler {
                binding: offset + 1,
                handle: self.sampler.clone(),
            },
        ];
    }
}
