use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::{Cursor, ErrorKind, Read, Write};

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
}

impl Packet {
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
        }

        Ok(bytes)
    }

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
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Trying to decode an invalid packet",
            )),
        }
    }
}
