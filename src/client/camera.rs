use cgmath::{perspective, Deg, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3};
use thunderdome::Index;
use winit::event::{ElementState, VirtualKeyCode};

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl Camera {
    pub fn new<P, Y>(position: P, yaw: Y, pitch: Y) -> Self
    where
        P: Into<Point3<f32>>,
        Y: Into<Rad<f32>>,
    {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn create_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            self.position,
            Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin()).normalize(),
            Vector3::unit_y(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Projection {
    aspect: f32,
    fov: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<T>(width: f32, height: f32, fov: T, znear: f32, zfar: f32) -> Self
    where
        T: Into<Rad<f32>>,
    {
        Self {
            aspect: width / height,
            fov: fov.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    pub fn create_projection(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fov, self.aspect, self.znear, self.zfar)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            proj: Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: Camera, projection: Projection) {
        self.proj = (projection.create_projection() * camera.create_matrix()).into();
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_down: f32,
    amount_up: f32,
    amount_back: f32,
    amount_forward: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.,
            amount_right: 0.,
            amount_down: 0.,
            amount_up: 0.,
            amount_back: 0.,
            amount_forward: 0.,
            rotate_horizontal: 0.,
            rotate_vertical: 0.,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {
        let amount = if let ElementState::Pressed = state {
            1.
        } else {
            0.
        };
        match key {
            VirtualKeyCode::W => self.amount_forward = amount,
            VirtualKeyCode::A => self.amount_left = amount,
            VirtualKeyCode::S => self.amount_back = amount,
            VirtualKeyCode::D => self.amount_right = amount,
            VirtualKeyCode::Space => self.amount_up = amount,
            VirtualKeyCode::LShift => self.amount_down = amount,
            _ => (),
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0., yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0., yaw_cos).normalize();

        camera.position += forward * (self.amount_forward - self.amount_back) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        self.rotate_horizontal = 0.;
        self.rotate_vertical = 0.;

        let max_pitch: Rad<f32> = Deg(90.).into();

        if camera.pitch < -max_pitch {
            camera.pitch = -max_pitch;
        } else if camera.pitch > max_pitch {
            camera.pitch = max_pitch;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraBundle {
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub projection: Projection,
    pub buffer: Index,
    pub bind_group: Index,
}
