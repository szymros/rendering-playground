const PI: f32 = 3.14159265359;
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_cords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) instance_transform_0: vec4<f32>,
    @location(5) instance_transform_1: vec4<f32>,
    @location(6) instance_transform_2: vec4<f32>,
    @location(7) instance_transform_3: vec4<f32>,
    @location(8) material_id: i32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_position: vec4<f32>,
    @location(3) @interpolate(flat) material_id: i32,
};

struct Camera {
    transform: mat4x4<f32>,
    origin: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
    color: vec4<f32>,
    pos: vec4<f32>,
    radius: f32,
    intensity: f32,
}

@group(1) @binding(0)
var<storage, read> light_buffer: array<Light>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let instance = mat4x4<f32>(in.instance_transform_0, in.instance_transform_1, in.instance_transform_2, in.instance_transform_3);
    let transformed = instance * vec4<f32>(in.position, 1.0);
    out.world_position = transformed;
    out.position = camera.transform * transformed;
    out.material_id = in.material_id;
    return out;
}

@group(2) @binding(0) 
var albedo: texture_2d<f32>;

@group(2) @binding(1)
var normals: texture_2d<f32>;

@group(2) @binding(2)
var frag_pos: texture_2d<f32>;

@group(2) @binding(3)
var material_tex: texture_2d<f32>;

// normal distribution function -> aproximates number of microfacets aligned to the halfway vector
// Trowbridge-Reitz
fn distribution_ggx(normal: vec3<f32>, half_way: vec3<f32>, roughness: f32) -> f32 {
    let roughness2 = roughness * roughness;
    let n_dot_h = max(dot(normal, half_way), 0.0);
    let n_dot_h2 = n_dot_h * n_dot_h;
    let nom = roughness2;
    var denom = (n_dot_h2 * (roughness2 - 1.0) + 1.0);
    denom = denom * denom * PI;
    return nom / denom;
}

//geometry function -> aproximates self shadowing of microfacets
fn geometry_schlick_ggx(n_dot_v: f32, k: f32) -> f32 {
    let nom = n_dot_v;
    let denom = n_dot_v * (1.0 - k) + k;
    return nom / denom;
}

fn geometry_smith(normal: vec3<f32>, view: vec3<f32>, light_dir: vec3<f32>, k: f32) -> f32 {
    let n_dot_v = max(dot(normal, view), 0.0);
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let ggx1 = geometry_schlick_ggx(n_dot_v, k);
    let ggx2 = geometry_schlick_ggx(n_dot_l, k);
    return ggx1 * ggx2;
}

//fresnel -> describes ratio of of light reflected to light refraced which varies over view angle
fn fresnel_schlick(n_dot_h: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - n_dot_h, 0.0, 1.0), 5.0);
}
@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureLoad(albedo, vec2<u32>(v_in.position.xy), 0).xyz;
    let normal = textureLoad(normals, vec2<u32>(v_in.position.xy), 0).xyz;
    let pos = textureLoad(frag_pos, vec2<u32>(v_in.position.xy), 0).xyz;
    let material = textureLoad(material_tex, vec2<u32>(v_in.position.xy), 0).xyz;
    let camera_pos = camera.origin.xyz;
    let light = light_buffer[v_in.material_id];
    let dist = length(light.pos.xyz - pos);
    let dist2 = dist * dist;
    let attenuation = max(min(1.0 - pow((dist / light.radius), 4), 1), 0) / dist2;
    let radiance = light.color.xyz * attenuation * light.intensity;

    let to_light = light.pos.xyz - pos;
    let light_dir = normalize(to_light);
    let view_dir = normalize(camera_pos - pos);
    let halfway = normalize(view_dir + light_dir);
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let n_dot_h = max(dot(normal, halfway), 0.0);
    let v_dot_h = max(dot(view_dir, halfway), 0.0);
    let metallic = material.b;
    var f0 = vec3(0.04);
    f0 = mix(f0, base_color, metallic);
    let f = fresnel_schlick(v_dot_h, f0);
    let roughness = material.g;
    let ndf = distribution_ggx(normal, halfway, roughness);
    let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let g = geometry_smith(normal, view_dir, light_dir, k);
    let numerator = ndf * g * f;
    let denominator = 4.0 * max(dot(normal, view_dir), 0.0) * max(dot(normal, light_dir), 0.0) + 0.0001;
    let specular = numerator / denominator;
    let ks = f;
    var kd = vec3<f32>(1.0) - ks;
    kd *= 1.0 - metallic;
    let color = (kd * base_color / PI + specular) * radiance * n_dot_l;
    //let hdr = color / (color + vec3<f32>(1.0));
    return vec4<f32>(color, 1.0);
}
