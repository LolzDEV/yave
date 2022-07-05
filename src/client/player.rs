use bevy_ecs::prelude::Component;
use winit::event::{ElementState, VirtualKeyCode};

pub struct PlayerController {
    pub amount_left: f32,
    pub amount_right: f32,
    pub amount_down: f32,
    pub amount_up: f32,
    pub amount_back: f32,
    pub amount_forward: f32,
    pub rotate_horizontal: f32,
    pub rotate_vertical: f32,
    pub speed: f32,
    pub sensitivity: f32,
}
impl PlayerController {
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
}

#[derive(Debug, Clone, Component)]
pub struct Player {
    pub name: String,
}
