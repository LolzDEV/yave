use crate::assets::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub data: u16,
    pub id: String,
}

impl Block {
    pub fn new(x: u16, y: u16, z: u16, id: Identifier, solid: bool) -> Self {
        Self {
            data: (x << 11) | ((y << 6) & 0x7c0) | ((z << 1) & 0x3e) | if solid { 1 } else { 0 },
            id: id.to_string(),
        }
    }

    pub fn x(&self) -> u8 {
        (&self.data >> 11) as u8
    }

    pub fn y(&self) -> u8 {
        ((&self.data & 0x7c0) >> 6) as u8
    }

    pub fn z(&self) -> u8 {
        ((&self.data & 0x3e) >> 1) as u8
    }
}

pub struct Chunk {
    pub blocks: Vec<Block>,
    pub x: i64,
    pub y: i64,
}

impl Chunk {
    pub fn new(x: i64, y: i64) -> Self {
        let blocks: Vec<Block> = (0..4096u16)
            .map(|mut i| {
                let z = i / (16 * 16);
                i -= z * 16 * 16;
                let y = i / 16;
                let x = i % 16;
                Block::new(x, y, z, Identifier::new("base", "stone"), true)
            })
            .collect();

        Self { blocks, x, y }
    }
}
