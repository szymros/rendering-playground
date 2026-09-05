struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_cords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) instance_transform_0: vec4<f32>,
    @location(5) instance_transform_1: vec4<f32>,
    @location(6) instance_transform_2: vec4<f32>,
    @location(7) instance_transform_3: vec4<f32>,
    @location(8)  material_id: i32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec4<f32>,
    @location(3) @interpolate(flat) material_id: i32,
    @location(4) tangent: vec3<f32>,
    @location(5) bitangent: vec3<f32>,
};

struct Camera {
    transform: mat4x4<f32>,
    origin: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var tex_array: binding_array<texture_2d<f32>>;
@group(1) @binding(1)
var tex_sampler: sampler;

struct Material {
    base_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    padding: vec2<u32>,
    emissive: vec3<f32>,
    base_color_texture_idx: i32,
    metallic_roughness_texture_idx: i32,
    normal_texture_idx: i32,
    occlusion_texture_idx: i32,
    emissive_texture_idx: i32,
    padding1: u32,
};

@group(2) @binding(0)
var<storage, read> materials: array<Material>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let instance = mat4x4<f32>(in.instance_transform_0, in.instance_transform_1, in.instance_transform_2, in.instance_transform_3);
    let transformed = instance * vec4<f32>(in.position, 1.0);
    let instance3x3 = mat3x3<f32>(in.instance_transform_0.xyz, in.instance_transform_1.xyz, in.instance_transform_2.xyz);
    out.world_position = transformed;
    out.position = camera.transform * transformed;
    out.uv = in.tex_cords;
    out.normal = instance3x3 * in.normal.xyz;
    out.material_id = in.material_id;
    out.tangent = instance3x3 * in.tangent.xyz;
    out.bitangent = cross(out.normal, out.tangent) * in.tangent.w;
    return out;
}

struct FragmentOut {
    @location(0) albedo: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) frag_pos: vec4<f32>,
    @location(3) material: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOut {
    let material = materials[u32(in.material_id)];
    var roughness = material.roughness;
    var metallic = material.metallic;
    if material.metallic_roughness_texture_idx > -1 {
        let tex = textureSample(tex_array[material.metallic_roughness_texture_idx], tex_sampler, in.uv);
        roughness *= tex.g;
        metallic *= tex.b;
    }
    let base_color_tex_idx = material.base_color_texture_idx;
    var base_color = material.base_color;
    if base_color_tex_idx > -1 {
        base_color *= textureSample(tex_array[base_color_tex_idx], tex_sampler, in.uv);
    }
    var normal = in.normal;
    if material.normal_texture_idx > -1 {
        let rgb_normal = textureSample(tex_array[material.normal_texture_idx], tex_sampler, in.uv);
        let tbn = mat3x3<f32>(normalize(in.tangent), normalize(in.bitangent), normalize(in.normal));
        normal = normalize(tbn * (rgb_normal.xyz * 2.0 - 1.0));
    }
    let out = FragmentOut(base_color, vec4<f32>(normal, 1.0), in.world_position, vec4<f32>(0.0, roughness, metallic, 1.0));
    return out;
}
