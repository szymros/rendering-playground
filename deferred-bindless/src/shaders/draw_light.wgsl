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
    out.uv = in.tex_cords;
    out.normal = in.normal;
    out.material_id = in.material_id;
    return out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let light = light_buffer[v_in.material_id];
    let color = vec4<f32>(light.color) * light.intensity;
    return vec4<f32>(color.xyz,1.0);
}
