use core::hash;
use std::borrow::Cow;

use gltf::{Document, image::Data, iter::Buffers};
use framework::{
    material::Material,
    mesh::{Mesh, MeshRegistry},
    resources::GpuResources,
    texture_array::TextureArray,
    vertex::Vertex,
};

use crate::scene::build_scene;

fn load_gltf_texture(
    gpu_resources: &mut GpuResources,
    texture_array: &mut TextureArray,
    image: &Data,
) -> u32 {
    let (converted_image, format) = match image.format {
        gltf::image::Format::R8G8B8 => {
            let rgba = image
                .pixels
                .chunks_exact(3)
                .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                .collect::<Vec<u8>>();
            (Cow::Owned(rgba), wgpu::TextureFormat::Rgba8Unorm)
        }
        gltf::image::Format::R16G16B16 => {
            let rgba = image
                .pixels
                .chunks_exact(6)
                .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], rgb[3], rgb[4], rgb[5], 255, 255])
                .collect::<Vec<u8>>();
            (Cow::Owned(rgba), wgpu::TextureFormat::Rgba16Unorm)
        }
        gltf::image::Format::R32G32B32FLOAT => {
            let rgba = image
                .pixels
                .chunks_exact(12)
                .flat_map(|rgb| {
                    [
                        rgb[0], rgb[1], rgb[2], rgb[3], rgb[4], rgb[5], rgb[6], rgb[7], rgb[8],
                        rgb[9], rgb[10], rgb[11], 255, 255, 255, 255,
                    ]
                })
                .collect::<Vec<u8>>();
            (Cow::Owned(rgba), wgpu::TextureFormat::Rgba32Float)
        }
        gltf::image::Format::R8G8B8A8 => (
            Cow::Borrowed(&image.pixels[..]),
            wgpu::TextureFormat::Rgba8Unorm,
        ),
        gltf::image::Format::R8 => (
            Cow::Borrowed(&image.pixels[..]),
            wgpu::TextureFormat::R8Unorm,
        ),
        gltf::image::Format::R8G8 => (
            Cow::Borrowed(&image.pixels[..]),
            wgpu::TextureFormat::Rg8Unorm,
        ),
        gltf::image::Format::R16 => (
            Cow::Borrowed(&image.pixels[..]),
            wgpu::TextureFormat::R16Unorm,
        ),
        gltf::image::Format::R16G16 => (
            Cow::Borrowed(&image.pixels[..]),
            wgpu::TextureFormat::Rg16Unorm,
        ),
        gltf::image::Format::R16G16B16A16 => (
            Cow::Borrowed(&image.pixels[..]),
            wgpu::TextureFormat::Rgba16Unorm,
        ),
        gltf::image::Format::R32G32B32A32FLOAT => (
            Cow::Borrowed(&image.pixels[..]),
            wgpu::TextureFormat::Rgba32Float,
        ),
    };
    let idx = texture_array.add_texture(
        &converted_image,
        (image.width, image.height),
        format,
        gpu_resources,
    );
    return idx;
}

pub fn load_gltf(
    path: &str,
    gpu_resources: &mut GpuResources,
    mesh_registry: &mut MeshRegistry,
    texture_array: &mut TextureArray,
) -> (Vec<(Mesh, u32)>, Vec<Material>) {
    let (document, buffers, images) = gltf::import(path).unwrap();
    let materials = load_materials(&document, &images, gpu_resources, texture_array);
    let mut meshes: Vec<(Mesh, u32)> = Vec::new();
    for gltf_mesh in document.meshes() {
        for primitive in gltf_mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let pos = reader
                .read_positions()
                .map(|pos| pos.collect::<Vec<[f32; 3]>>())
                .unwrap_or(vec![]);
            let normals = reader
                .read_normals()
                .map(|normal| normal.collect::<Vec<[f32; 3]>>())
                .unwrap_or(vec![[0.0; 3]; pos.len()]);
            let tex_coords = reader
                .read_tex_coords(0)
                .map(|tex_coords| tex_coords.into_f32().collect::<Vec<[f32; 2]>>())
                .unwrap_or(vec![[0.0; 2]; pos.len()]);
            let tangents = reader
                .read_tangents()
                .map(|tangent| tangent.collect::<Vec<[f32; 4]>>())
                .unwrap_or(vec![[0.0; 4]; pos.len()]);
            let indices = reader
                .read_indices()
                .map(|indices| indices.into_u32().collect::<Vec<u32>>())
                .unwrap_or(vec![]);
            let vertices: Vec<Vertex> = (0..pos.len())
                .map(|i| Vertex {
                    pos: pos[i],
                    tex_cords: tex_coords[i],
                    normals: normals[i],
                    tangent: tangents[i],
                })
                .collect();
            let mesh = mesh_registry.upload(&vertices, &indices);
            let material_id = primitive.material().index().unwrap_or(0) as u32;
            meshes.push((mesh, material_id));
        }
    }
    return (meshes, materials);
}

pub fn load_materials(
    document: &Document,
    images: &Vec<Data>,
    gpu_resources: &mut GpuResources,
    texture_array: &mut TextureArray,
) -> Vec<Material> {
    let mut materials: Vec<Material> = Vec::new();
    for material in document.materials() {
        let params = material.pbr_metallic_roughness();
        let base_color_tex: i32 =
            if let Some(texture) = material.pbr_metallic_roughness().base_color_texture() {
                let image_idx = texture.texture().source().index();
                load_gltf_texture(gpu_resources, texture_array, &images[image_idx]) as i32
            } else {
                -1
            };
        let metallic_roughness_tex: i32 = if let Some(texture) = material
            .pbr_metallic_roughness()
            .metallic_roughness_texture()
        {
            let image_idx = texture.texture().source().index();
            load_gltf_texture(gpu_resources, texture_array, &images[image_idx]) as i32
        } else {
            -1
        };
        let normals_tex: i32 = if let Some(texture) = material.normal_texture() {
            let image_idx = texture.texture().source().index();
            load_gltf_texture(gpu_resources, texture_array, &images[image_idx]) as i32
        } else {
            -1
        };
        let occlusion_tex: i32 = if let Some(texture) = material.occlusion_texture() {
            let image_idx = texture.texture().source().index();
            load_gltf_texture(gpu_resources, texture_array, &images[image_idx]) as i32
        } else {
            -1
        };
        let emissive_tex: i32 = if let Some(texture) = material.emissive_texture() {
            let image_idx = texture.texture().source().index();
            load_gltf_texture(gpu_resources, texture_array, &images[image_idx]) as i32
        } else {
            -1
        };
        materials.push(Material {
            base_color: params.base_color_factor(),
            metallic: params.metallic_factor(),
            roughness: params.metallic_factor(),
            emissive: [0.0; 3],
            base_color_texture_idx: base_color_tex,
            metallic_roughness_texture_idx: metallic_roughness_tex,
            normal_texture_idx: normals_tex,
            occlusion_texture_idx: occlusion_tex,
            emissive_texture_idx: emissive_tex,
            _padding: [0; 2],
            _padding1: 0,
        });
    }
    return materials;
}

pub fn load_mesh_from_obj(file_path: &str, mesh_registry: &mut MeshRegistry) -> Vec<Mesh> {
    let (loaded_models, _) = tobj::load_obj(
        file_path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .unwrap();
    let mut meshes: Vec<Mesh> = Vec::new();
    for model in loaded_models.iter() {
        let vertices: Vec<Vertex> = (0..model.mesh.positions.len() / 3)
            .map(|i| Vertex {
                pos: [
                    model.mesh.positions[i * 3],
                    model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                ],
                tex_cords: [
                    model.mesh.texcoords[i * 2],
                    1.0 - model.mesh.texcoords[i * 2 + 1],
                ],
                normals: [
                    model.mesh.normals[i * 3],
                    model.mesh.normals[i * 3 + 1],
                    model.mesh.normals[i * 3 + 2],
                ],
                tangent: [0.0, 0.0, 0.0, 0.0],
            })
            .collect();
        let mesh = mesh_registry.upload(&vertices, &model.mesh.indices);
        meshes.push(mesh);
    }
    return meshes;
}
