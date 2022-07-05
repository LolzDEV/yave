extern crate core;

use std::time::Duration;
use winit::event::KeyboardInput;

pub mod assets;
pub mod client;
pub mod network;
pub mod server;
pub mod world;

pub struct DeltaTime(Duration);

pub struct KeyboardEvent {
    pub input: KeyboardInput,
}

pub struct MouseMotion {
    pub delta: (f64, f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Negative Z
    North,
    /// Positive Z
    South,
    /// Positive X
    East,
    /// Negative X
    West,
    /// Positive Y
    Top,
    /// Negative Y
    Bottom,
}
