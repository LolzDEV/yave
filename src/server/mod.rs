use crate::network::Packet;
use bevy_ecs::prelude::Component;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;

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

pub struct SocketSender {
    socket: Arc<tokio::net::UdpSocket>,
}

pub struct SocketReceiver {
    socket: Arc<tokio::net::UdpSocket>,
}

impl SocketSender {
    pub async fn send_to(&mut self, bytes: &[u8], addr: &SocketAddr) -> io::Result<()> {
        self.socket.send_to(bytes, addr).await?;
        Ok(())
    }

    pub async fn send(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.socket.send(bytes).await?;
        Ok(())
    }
}

impl SocketReceiver {
    pub async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf).await
    }

    pub async fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf).await
    }
}

pub fn split_socket(socket: UdpSocket) -> (SocketSender, SocketReceiver) {
    let arc = Arc::new(tokio::net::UdpSocket::from_std(socket).unwrap());

    (
        SocketSender {
            socket: arc.clone(),
        },
        SocketReceiver {
            socket: arc.clone(),
        },
    )
}
