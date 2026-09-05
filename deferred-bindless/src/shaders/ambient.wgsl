struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) 
var albedo: texture_2d<f32>;

@group(0) @binding(1)
var normals: texture_2d<f32>;

@group(0) @binding(2)
var frag_pos: texture_2d<f32>;

@group(0) @binding(3)
var material_tex: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // fullscreen triangle 
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );
    out.position = vec4<f32>(pos[vertex_index], 0.0, 1.0);

    // convert clip-space → UV
    out.uv = out.position.xy * 0.5 + vec2<f32>(0.5);
    return out;
}

@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureLoad(albedo, vec2<u32>(v_in.position.xy), 0).xyz;
    return vec4<f32>(base_color * 0.1, 1.0);
}
