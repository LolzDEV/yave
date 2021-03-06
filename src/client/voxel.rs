use serde_derive::Deserialize;

use crate::Direction;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub struct VoxelVertex {
    data: u32,
}

impl VoxelVertex {
    /// Vertex data is compressed into a single 32 bit unsigned integer to save video memory
    pub fn new(x: u32, y: u32, z: u32, texture_index: u32, atlas_index: u32) -> Self {
        Self {
            data: (x << 27)
                | ((y << 22) & 0x7c00000)
                | ((z << 17) & 0x3e0000)
                | ((texture_index & 0x7) << 15)
                | (atlas_index & 0xF),
        }
    }

    pub fn x(&self) -> u32 {
        self.data >> 27
    }

    pub fn y(&self) -> u32 {
        (self.data & 0x7c00000) >> 22
    }

    pub fn z(&self) -> u32 {
        (self.data & 0x3e0000) >> 17
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VoxelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Uint32,
            }],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockDescription {
    pub solid: bool,
    pub texture: BlockTextureDescription,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockTextureDescription {
    pub top: String,
    pub bottom: String,
    pub north: String,
    pub west: String,
    pub east: String,
    pub south: String,
}

#[derive(Debug, Clone)]
pub struct BlockFace {
    pub vertices: [VoxelVertex; 4],
}

impl BlockFace {
    pub fn new(direction: Direction, x: u32, y: u32, z: u32) -> Self {
        match direction {
            Direction::North => Self {
                vertices: [
                    VoxelVertex::new(0 + x, 0 + y, 0 + z, 0, 0),
                    VoxelVertex::new(1 + x, 0 + y, 0 + z, 0, 0),
                    VoxelVertex::new(1 + x, 1 + y, 0 + z, 0, 0),
                    VoxelVertex::new(0 + x, 1 + y, 0 + z, 0, 0),
                ],
            },
            Direction::South => Self {
                vertices: [
                    VoxelVertex::new(0 + x, 0 + y, 1 + z, 0, 0),
                    VoxelVertex::new(1 + x, 0 + y, 1 + z, 0, 0),
                    VoxelVertex::new(1 + x, 1 + y, 1 + z, 0, 0),
                    VoxelVertex::new(0 + x, 1 + y, 1 + z, 0, 0),
                ],
            },
            Direction::East => Self {
                vertices: [
                    VoxelVertex::new(1 + x, 0 + y, 1 + z, 0, 0),
                    VoxelVertex::new(1 + x, 0 + y, 0 + z, 0, 0),
                    VoxelVertex::new(1 + x, 1 + y, 0 + z, 0, 0),
                    VoxelVertex::new(1 + x, 1 + y, 1 + z, 0, 0),
                ],
            },
            Direction::West => Self {
                vertices: [
                    VoxelVertex::new(0 + x, 0 + y, 1 + z, 0, 0),
                    VoxelVertex::new(0 + x, 0 + y, 0 + z, 0, 0),
                    VoxelVertex::new(0 + x, 1 + y, 0 + z, 0, 0),
                    VoxelVertex::new(0 + x, 1 + y, 1 + z, 0, 0),
                ],
            },
            Direction::Top => Self {
                vertices: [
                    VoxelVertex::new(0 + x, 1 + y, 0 + z, 0, 0),
                    VoxelVertex::new(1 + x, 1 + y, 0 + z, 0, 0),
                    VoxelVertex::new(1 + x, 1 + y, 1 + z, 0, 0),
                    VoxelVertex::new(0 + x, 1 + y, 1 + z, 0, 0),
                ],
            },
            Direction::Bottom => Self {
                vertices: [
                    VoxelVertex::new(0 + x, 0 + y, 1 + z, 0, 0),
                    VoxelVertex::new(1 + x, 0 + y, 1 + z, 0, 0),
                    VoxelVertex::new(1 + x, 0 + y, 0 + z, 0, 0),
                    VoxelVertex::new(0 + x, 0 + y, 0 + z, 0, 0),
                ],
            },
        }
    }
}
