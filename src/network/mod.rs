use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use pollster::block_on;
use std::io;
use std::io::{Cursor, ErrorKind, Read, Write};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;

use crate::world::chunk::BlockGroup;

#[derive(Debug, Clone)]
pub enum Packet {
    /// Connection packet. This is sent by the client to the server when a connection is enstablished.
    Connection { user: String },
    /// Player movement. This is sent by the client to the server when the position needs to be changed.
    Movement {
        delta_x: f64,
        delta_y: f64,
        delta_z: f64,
    },
    /// Position request. This is sent by the client to the server when it wants to know the new position.
    PositionRequest { name: String },
    /// Player position. This is sent by the server to the client when requested.
    PlayerPosition {
        x: f64,
        y: f64,
        z: f64,
        name: String,
    },
    /// Online player list. Sent by the server to the client when a new client connects.
    OnlinePlayers { players: Vec<OnlinePlayer> },
    /// Unload chunk. Sent by the server to the client when a chunk is unloaded.
    UnloadChunk { x: i64, y: i64 },
    /// Chunk. Sent by the server to the client when a new chunk is loaded.
    Chunk {
        x: i64,
        y: i64,
        groups: Vec<BlockGroup>,
    },
}

/// Structure used in the OnlinePlayers packet to store information about players.
#[derive(Debug, Clone)]
pub struct OnlinePlayer {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Packet {
    /// Encode a packet into bytes to send it over the internet.
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();

        match self {
            Packet::Connection { user } => {
                bytes.write_u8(0)?;
                bytes.write_u64::<BigEndian>(user.len() as u64)?;
                bytes.write_all(user.as_bytes())?;
            }
            Packet::Movement {
                delta_x,
                delta_y,
                delta_z,
            } => {
                bytes.write_u8(1)?;
                bytes.write_f64::<BigEndian>(*delta_x)?;
                bytes.write_f64::<BigEndian>(*delta_y)?;
                bytes.write_f64::<BigEndian>(*delta_z)?;
            }
            Packet::PositionRequest { name } => {
                bytes.write_u8(2)?;
                bytes.write_u64::<BigEndian>(name.len() as u64)?;
                bytes.write_all(name.as_bytes())?;
            }
            Packet::PlayerPosition { x, y, z, name } => {
                bytes.write_u8(3)?;
                bytes.write_f64::<BigEndian>(*x)?;
                bytes.write_f64::<BigEndian>(*y)?;
                bytes.write_f64::<BigEndian>(*z)?;
                bytes.write_u64::<BigEndian>(name.len() as u64)?;
                bytes.write_all(name.as_bytes())?;
            }
            Packet::OnlinePlayers { players } => {
                bytes.write_u8(4)?;
                bytes.write_u64::<BigEndian>(players.len() as u64)?;
                for player in players {
                    bytes.write_u64::<BigEndian>(player.name.len() as u64)?;
                    bytes.write_all(player.name.as_bytes())?;

                    bytes.write_f32::<BigEndian>(player.x)?;
                    bytes.write_f32::<BigEndian>(player.y)?;
                    bytes.write_f32::<BigEndian>(player.z)?;
                }
            }
            Packet::UnloadChunk { x, y } => {
                bytes.write_u8(5)?;
                bytes.write_i64::<BigEndian>(*x)?;
                bytes.write_i64::<BigEndian>(*y)?;
            }
            Packet::Chunk { x, y, groups } => {
                bytes.write_u8(6)?;
                bytes.write_i64::<BigEndian>(*x)?;
                bytes.write_i64::<BigEndian>(*y)?;
                bytes.write_u64::<BigEndian>(groups.len() as u64)?;
                for group in groups {
                    bytes.write_u64::<BigEndian>(group.id.len() as u64)?;
                    bytes.write_all(group.id.as_bytes())?;
                    bytes.write_u32::<BigEndian>(group.count)?;
                }
            }
        }

        Ok(bytes)
    }

    /// Decode a received packet from bytes.
    pub fn decode(data: Vec<u8>) -> io::Result<Self> {
        let mut cursor = Cursor::new(data);
        let id = cursor.read_u8()?;

        match id {
            0 => {
                let size = cursor.read_u64::<BigEndian>()? as usize;
                let mut buf = vec![0u8; size];
                cursor.read_exact(&mut buf)?;
                Ok(Self::Connection {
                    user: String::from_utf8(buf).unwrap_or_else(|_| String::from("invalid")),
                })
            }
            1 => Ok(Self::Movement {
                delta_x: cursor.read_f64::<BigEndian>()?,
                delta_y: cursor.read_f64::<BigEndian>()?,
                delta_z: cursor.read_f64::<BigEndian>()?,
            }),
            2 => {
                let size = cursor.read_u64::<BigEndian>()? as usize;
                let mut buf = vec![0u8; size];
                cursor.read_exact(&mut buf)?;
                Ok(Self::PositionRequest {
                    name: String::from_utf8(buf).unwrap_or_else(|_| String::from("invalid")),
                })
            }
            3 => {
                let x = cursor.read_f64::<BigEndian>()?;
                let y = cursor.read_f64::<BigEndian>()?;
                let z = cursor.read_f64::<BigEndian>()?;

                let size = cursor.read_u64::<BigEndian>()? as usize;
                let mut buf = vec![0u8; size];
                cursor.read_exact(&mut buf)?;
                let name = String::from_utf8(buf).unwrap_or_else(|_| String::from("Invalid"));

                Ok(Self::PlayerPosition { x, y, z, name })
            }
            4 => {
                let len = cursor.read_u64::<BigEndian>()?;
                let mut players = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let name_len = cursor.read_u64::<BigEndian>()?;

                    let mut buf = vec![0u8; name_len as usize];
                    cursor.read_exact(&mut buf)?;

                    let name = String::from_utf8(buf).unwrap_or_else(|_| String::from("Invalid"));
                    let x = cursor.read_f32::<BigEndian>()?;
                    let y = cursor.read_f32::<BigEndian>()?;
                    let z = cursor.read_f32::<BigEndian>()?;

                    players.push(OnlinePlayer { name, x, y, z });
                }

                Ok(Self::OnlinePlayers { players })
            }
            5 => Ok(Self::UnloadChunk {
                x: cursor.read_i64::<BigEndian>()?,
                y: cursor.read_i64::<BigEndian>()?,
            }),
            6 => {
                let x = cursor.read_i64::<BigEndian>()?;
                let y = cursor.read_i64::<BigEndian>()?;
                let groups_size = cursor.read_u64::<BigEndian>()?;

                let mut groups = Vec::new();

                for _ in 0..groups_size {
                    let id_len = cursor.read_u64::<BigEndian>()?;

                    let mut buf = vec![0u8; id_len as usize];
                    cursor.read_exact(&mut buf)?;

                    let id = String::from_utf8(buf).unwrap_or_else(|_| String::from("Invalid"));

                    let count = cursor.read_u32::<BigEndian>()?;

                    groups.push(BlockGroup { id, count });
                }

                Ok(Self::Chunk { x, y, groups })
            }
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Trying to decode an invalid packet",
            )),
        }
    }
}

/// SocketSender is used with a SocketReceiver to split a UdpSocket into two indipendent parts. This is generated in pair using the split_socket function
pub struct SocketSender {
    socket: Arc<tokio::net::UdpSocket>,
}

/// SocketReceiver is used with a SocketSender to split a UdpSocket into two indipendent parts. This is generated in pair using the split_socket function
pub struct SocketReceiver {
    socket: Arc<tokio::net::UdpSocket>,
}

impl SocketSender {
    /// Send a packet to the specified socket.
    pub fn send_to(&mut self, packet: Packet, addr: &SocketAddr) -> io::Result<()> {
        block_on(self.socket.send_to(packet.encode()?.as_slice(), addr))?;
        Ok(())
    }

    /// Send a packet to the connected socket.
    pub fn send(&mut self, packet: Packet) -> io::Result<()> {
        block_on(self.socket.send(packet.encode()?.as_slice()))?;
        Ok(())
    }
}

impl SocketReceiver {
    pub fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        block_on(self.socket.recv_from(buf))
    }

    #[allow(dead_code)]
    pub async fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        block_on(self.socket.recv(buf))
    }
}

/// Split a UdpSocket into two indipendent parts, a sender and a receiver.
pub fn split_socket(socket: UdpSocket) -> (SocketSender, SocketReceiver) {
    let arc = Arc::new(tokio::net::UdpSocket::from_std(socket).unwrap());

    (
        SocketSender {
            socket: arc.clone(),
        },
        SocketReceiver { socket: arc },
    )
}
