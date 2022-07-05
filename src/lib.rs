use std::time::Duration;
use winit::event::KeyboardInput;

pub mod assets;
pub mod client;
mod network;
pub mod server;
pub mod world;

pub struct DeltaTime(Duration);

pub struct KeyboardEvent {
    pub input: KeyboardInput,
}

pub struct MouseMotion {
    pub delta: (f64, f64),
}
