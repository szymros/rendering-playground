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
    instance_count: atomic<u32>,
    _pad: vec2<u32>,
    origin: vec2<f32>,
    to_emit: u32,
    params: array<u32,4>,
    dead_indices_count: atomic<i32>,
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

@group(1) @binding(3)
var<storage, read_write> alive_indicies: array<u32>;

struct IndirectDrawArgs {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_index: u32,
    first_instance: u32,
}

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
    var particle = particles[id.x];
    if particle.age < particle.lifetime {
        var seed = globals.seed ^ bitcast<u32>(globals.time_delta) ^ id.x;
        let alive_idx = atomicAdd(&locals.instance_count, 1);
        alive_indicies[alive_idx] = id.x;

        {{ UPDATE }}

        // INTEGRATE
        particle.age += globals.time_delta;
        particle.pos += particle.velocity * globals.time_delta;
        particle.velocity += particle.acceleration * globals.time_delta;
        particles[id.x] = particle;
        if particle.age > particle.lifetime {
            let idx = atomicAdd(&locals.dead_indices_count, 1);
            dead_indicies[idx] = id.x;
            return;
        }
    }
}
