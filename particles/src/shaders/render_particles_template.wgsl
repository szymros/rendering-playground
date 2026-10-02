struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) particle_index: u32,
};

struct Globals {
    proj: mat4x4<f32>,
    seed: u32,
    time_delta: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct Particle {
    pos: vec2<f32>,
    velocity: vec2<f32>,
    acceleration: vec2<f32>,
    color: u32,
    age: f32,
    sprite_idx: u32,
    lifetime: f32,
}

@group(1) @binding(0)
var<storage, read> particles: array<Particle>;

@group(1) @binding(2)
var<storage, read> alive_indcies: array<u32>;

{{ ATLAS_BINDS }}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let particle_index = alive_indcies[instance_index];
    let partcile = particles[particle_index];
    //quad
    let vertices = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5),
        vec2<f32>(-0.5, -0.5), vec2<f32>(-0.5, 0.5), vec2<f32>(0.5, 0.5),
    );
    let vertex = vertices[vertex_index];
    let world_pos = partcile.pos.xy +  vertex * {{ PARTICLE_SIZE }};
    let projected = globals.proj * vec4<f32>(world_pos, 0.0, 1.0);

    out.position = projected;

    out.uv = vertex + vec2<f32>(0.5);
    out.particle_index = particle_index;
    return out;
}



@fragment
fn fs_main(v_in: VertexOutput) -> @location(0) vec4<f32> {
    let particle = particles[v_in.particle_index];
    var color = unpack4x8unorm(particle.color);
    {{ RENDER }}
    if all(color.xyz == vec3<f32>(0.0)) {
        discard;
    }
    return color;
}
