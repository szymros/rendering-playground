use std::collections::HashMap;

use glam::{Quat, Vec3};
use framework::{instance::Instance, mesh::Mesh};

use crate::light::Light;

struct SceneNode {
    children: Vec<u32>,
    instance_id: u32,
}

struct Scene {}

pub fn build_scene() {
    let (document, buffers, images) = gltf::import("./assets/helmet/SciFiHelmet.gltf").unwrap();
    let instance_for_mesh: HashMap<u32, Vec<Instance>> = HashMap::new();
    let scene = document.default_scene().unwrap();
    let nodes: Vec<SceneNode> = Vec::new();
    for node in scene.nodes() {
        let instance = match node.transform() {
            gltf::scene::Transform::Matrix { matrix } => Instance {
                transform: matrix,
                material_id: -1,
            },
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => Instance::from_translation_rotation_scale(
                Vec3::from_array(translation),
                Quat::from_array(rotation),
                Vec3::from_array(scale),
            ),
        };
    }
}
