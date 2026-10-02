fn pcg(v: u32) -> u32 {
    let state = v * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_pcg(rng_state: ptr<function,u32>) -> u32 {
    let state = *rng_state;
    *rng_state = *rng_state * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return word;
}

fn rand(state: ptr<function,u32>) -> f32 {
    let x = rand_pcg(state);
    *state = x;
    return f32(x) * bitcast<f32>(0x2f800000u);
}

struct Locals {
    vertex_count: u32,
    instance_count: u32,
    pad: vec2<u32>,
    origin: vec2<f32>,
    to_emit: u32,
    params: array<u32,4>,
    dead_indicies_count: atomic<i32>,
}
@group(1) @binding(0)
var<storage, read_write> locals: Locals;

struct Particle {
    pos: vec2<f32>,
    velocity: vec2<f32>,
    acceleration: vec2<f32>,
    color: u32,
    age: f32,
    sprite_idx: u32,
    lifetime: f32,
}

@group(1) @binding(1)
var<storage, read_write> particles: array<Particle>;

@group(1) @binding(2)
var<storage, read_write> dead_indicies: array<u32>;

struct Globals {
    proj: mat4x4<f32>,
    seed: u32,
    time_delta: f32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@compute @workgroup_size(16)
fn main(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    if id.x > locals.to_emit {
        return;
    }
    let dead_idx = atomicSub(&locals.dead_indicies_count, 1) - 1;
    if dead_idx < 0 {
        atomicAdd(&locals.dead_indicies_count, 1);
        return;
    }
    let particle_index = dead_indicies[dead_idx];
    var seed = globals.seed ^ bitcast<u32>(globals.time_delta) ^ particle_index;
    var particle: Particle;
    {{ INIT }}
    particles[particle_index] = particle;
}
