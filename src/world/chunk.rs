use std::str::FromStr;

use bevy_ecs::prelude::Component;

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

    pub fn transparent(&self) -> bool {
        (&self.data & 0x1) == 0
    }
}

#[derive(Debug, Clone, Component)]
pub struct Chunk {
    pub blocks: Vec<Block>,
    pub x: i64,
    pub y: i64,
}

impl Chunk {
    /// Generate a new chunk at given position.
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

    /// Compress a chunk in a smaller data structure.
    pub fn compress(&self) -> Vec<BlockGroup> {
        let mut groups = Vec::new();

        let mut group = BlockGroup {
            id: self.blocks.first().unwrap().id.clone(),
            count: 0,
        };

        for block in self.blocks.iter() {
            if block.id == group.id {
                group.count += 1;
            } else {
                groups.push(group);

                group = BlockGroup {
                    id: block.id.clone(),
                    count: 1,
                };
            }
        }

        groups.push(group);

        groups
    }

    /// Decomrpess a chunk from a vector of BlockGroups.
    pub fn decompress(groups: &Vec<BlockGroup>, x: i64, y: i64) -> Self {
        let mut chunk = Self {
            blocks: Vec::new(),
            x,
            y,
        };

        let mut i = 0i16;

        for group in groups {
            for _ in 0..group.count {
                let z = (i / (16 * 16)) as u16;
                let n_i = i - (z as i16 * 16 * 16);
                let y = (n_i / 16) as u16;
                let x = (n_i % 16) as u16;

                let block = Block::new(x, y, z, Identifier::from_str(&group.id).unwrap(), true);

                chunk.blocks.push(block);

                i += 1;
            }
        }

        chunk
    }

    pub fn get_block(&self, x: u16, y: u16, z: u16) -> Option<&Block> {
        let i = (z * 16 * 16) + (y * 16) + x;
        self.blocks.get(i as usize)
    }
}

/// Data structure used to represent a compressed chunk.
#[derive(Debug, Clone)]
pub struct BlockGroup {
    pub id: String,
    pub count: u32,
}
