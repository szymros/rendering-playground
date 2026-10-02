
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pos: [f32; 2],
    velocity: [f32; 2],
    acceleration: [f32; 2],
    color: u32,
    life: f32,
    sprite_idx: u32,
    age: f32,
}

