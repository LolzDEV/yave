use crate::network::Packet;

pub mod camera;
pub mod chunk;
pub mod game;
pub mod player;
pub mod renderer;
pub mod transform;
pub mod voxel;

/// This is an event sent by the network handler in the client when the server sends a new packet.
pub struct ServerEvent {
    pub packet: Packet,
}
