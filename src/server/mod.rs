use crate::network::Packet;
use bevy_ecs::prelude::Component;
use std::net::SocketAddr;

pub mod game;

#[derive(Debug, Clone, Component)]
pub struct PlayerName {
    pub name: String,
}

#[derive(Debug, Clone, Component)]
pub struct ClientEvent {
    pub packet: Packet,
    pub peer: SocketAddr,
}

#[derive(Debug, Clone, Component)]
pub struct Player {
    pub name: PlayerName,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Component)]
pub struct Connection {
    pub peer: SocketAddr,
}
