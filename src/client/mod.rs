use crate::network::Packet;

pub mod camera;
pub mod chunk;
pub mod game;
pub mod player;
pub mod renderer;
pub mod transform;
pub mod voxel;

pub struct ServerEvent {
    pub packet: Packet,
}
