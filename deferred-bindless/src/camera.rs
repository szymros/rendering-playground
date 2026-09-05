use glam::{
    Mat4, Quat, Vec3,
    camera::rh::{proj::directx::perspective, view::look_to_mat4},
};
use framework::{
    input::{InputState, Key},
    resources::{BufferHandle, GpuResources, UNIFORM_BUFFER_USAGES},
};

const PITCH_LIMIT: f32 = 1.54;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub transform: [[f32; 4]; 4],
    pub position: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub origin: Vec3,
    pub forward: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub view: Mat4,
    pub velocity: Vec3,
    pub buffer_handle: BufferHandle,
    pub uniform: CameraUniform,
    proj: Mat4,
}

impl Camera {
    pub fn new(
        origin: Vec3,
        gpu_resources: &mut GpuResources,
        screen_size: (u32, u32),
    ) -> Camera {
        let forward = Vec3::Z;
        let buffer_handle = gpu_resources.create_buffer(
            std::mem::size_of::<CameraUniform>() as u64,
            "camera_uniform_buffer",
            UNIFORM_BUFFER_USAGES,
        );
        let view = look_to_mat4(origin, forward, Vec3::Y);
        let proj = perspective(
            f32::to_radians(60.0),
            screen_size.0 as f32 / screen_size.1 as f32,
            0.1,
            100.0,
        );
        let view_proj = proj * view;
        let uniform = CameraUniform {
            transform: view_proj.to_cols_array_2d(),
            position: origin.extend(0.0).to_array(),
        };

        return Camera {
            origin,
            forward,
            velocity: Vec3::ZERO,
            pitch: 0.0,
            yaw: 0.0,
            uniform,
            buffer_handle,
            view,
            proj,
        };
    }

    pub fn update_buffer(
        &self,
        uniform: CameraUniform,
        queue: &wgpu::Queue,
        gpu_resources: &GpuResources,
    ) {
        let buffer = gpu_resources
            .buffers
            .get(&self.buffer_handle)
            .expect("dangling handle for camera");
        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn recalculate_uniform(&mut self) {
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, self.yaw);
        let pitch_rot = Quat::from_axis_angle(Vec3::X, self.pitch);
        let total_rot = yaw_rot * pitch_rot;

        self.forward = (total_rot * Vec3::Z).normalize();
        self.view = look_to_mat4(self.origin, self.forward, Vec3::Y);
        self.uniform = CameraUniform {
            transform: (self.proj * self.view).to_cols_array_2d(),
            position: self.origin.extend(0.0).to_array(),
        };
    }
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    return start + (end - start) * t;
}

pub struct CameraController {
    previous_state: Camera,
    acceleration: f32,
    deacceleration: f32,
    max_speed: f32,
    mouse_senitivity: f32,
}

impl CameraController {
    pub fn new(camera: Camera) -> CameraController {
        return CameraController {
            previous_state: camera,
            acceleration: 50.0,
            deacceleration: 20.0,
            max_speed: 4.0,
            mouse_senitivity: 0.005,
        };
    }

    pub fn rotate(camera: &mut Camera, mouse_delta: (f64, f64), mouse_sens: f32) {
        camera.yaw -= mouse_delta.0 as f32 * mouse_sens;
        camera.pitch += mouse_delta.1 as f32 * mouse_sens;
        camera.pitch = camera.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, camera.yaw);
        let pitch_rot = Quat::from_axis_angle(Vec3::X, camera.pitch);
        let total_rot = yaw_rot * pitch_rot;
        camera.forward = (total_rot * Vec3::Z).normalize();
    }

    pub fn update_camera(
        &mut self,
        camera: &mut Camera,
        input_state: &InputState,
        delta_time: f32,
    ) {
        self.previous_state = camera.clone();
        CameraController::rotate(camera, input_state.mouse_delta, self.mouse_senitivity);
        let mut dir = Vec3::ZERO;
        if input_state.held.contains(&Key::KeyW) {
            dir += camera.forward;
        };
        if input_state.held.contains(&Key::KeyS) {
            dir += -camera.forward;
        };
        if input_state.held.contains(&Key::KeyA) {
            dir += -camera.view.row(0).truncate();
        };
        if input_state.held.contains(&Key::KeyD) {
            dir += camera.view.row(0).truncate();
        };
        dir = dir.normalize_or_zero();
        let target_velocity = dir * self.max_speed;
        let rate = if dir != Vec3::ZERO {
            self.acceleration
        } else {
            self.deacceleration
        };

        let mut velocity = camera.velocity.lerp(target_velocity, delta_time * rate);
        if velocity.length_squared() < 0.001 {
            velocity = Vec3::ZERO;
        }
        camera.velocity = velocity;
        camera.origin += camera.velocity * delta_time;
        camera.recalculate_uniform();
    }

    pub fn interpolate(&self, current_state: &Camera, time_alpha: f32) -> Camera {
        let mut interpolated = self.previous_state.clone();
        interpolated.origin = interpolated.origin.lerp(current_state.origin, time_alpha);
        interpolated.velocity = interpolated
            .velocity
            .lerp(current_state.velocity, time_alpha);
        interpolated.pitch = lerp(interpolated.pitch, current_state.pitch, time_alpha);
        interpolated.yaw = lerp(interpolated.yaw, current_state.yaw, time_alpha);
        interpolated.recalculate_uniform();
        return interpolated;
    }
}
