use crate::client::renderer::Renderer;
use crate::client::voxel::VoxelVertex;
use crate::world::chunk::Chunk;
use crate::Direction;
use bevy_ecs::prelude::Component;
use thunderdome::Index;
use wgpu::BufferUsages;

use super::voxel::BlockFace;

#[derive(Debug, Clone, PartialEq, Component)]
pub struct ChunkMesh {
    pub buffer: Index,
}

impl ChunkMesh {
    pub fn new(renderer: &mut Renderer, vertices: Vec<VoxelVertex>) -> Self {
        Self {
            buffer: renderer.create_buffer(
                bytemuck::cast_slice(vertices.as_slice()),
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            ),
        }
    }

    pub fn build(renderer: &mut Renderer, chunk: &Chunk) -> Self {
        let mut vertices: Vec<VoxelVertex> = Vec::new();

        // Generate vertices.
        for i in 0..4096 {
            let block = chunk.blocks.get(i).unwrap();
            let x = block.x() as u16;
            let y = block.y() as u16;
            let z = block.z() as u16;

            if y == 15 {
                vertices.append(
                    &mut BlockFace::new(Direction::Top, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            } else if let Some(block) = chunk.get_block(x, y + 1, z) {
                if block.transparent() {
                    vertices.append(
                        &mut BlockFace::new(Direction::Top, x as u32, y as u32, z as u32)
                            .vertices
                            .to_vec(),
                    );
                }
            } else {
                vertices.append(
                    &mut BlockFace::new(Direction::Top, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            }

            if y == 0 {
                vertices.append(
                    &mut BlockFace::new(Direction::Bottom, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            } else if let Some(block) = chunk.get_block(x, y - 1, z) {
                if block.transparent() {
                    vertices.append(
                        &mut BlockFace::new(Direction::Bottom, x as u32, y as u32, z as u32)
                            .vertices
                            .to_vec(),
                    );
                }
            } else {
                vertices.append(
                    &mut BlockFace::new(Direction::Bottom, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            }

            if x == 15 {
                vertices.append(
                    &mut BlockFace::new(Direction::East, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            } else if let Some(block) = chunk.get_block(x + 1, y, z) {
                if block.transparent() {
                    vertices.append(
                        &mut BlockFace::new(Direction::East, x as u32, y as u32, z as u32)
                            .vertices
                            .to_vec(),
                    );
                }
            } else {
                vertices.append(
                    &mut BlockFace::new(Direction::East, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            }

            if x == 0 {
                vertices.append(
                    &mut BlockFace::new(Direction::West, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            } else if let Some(block) = chunk.get_block(x - 1, y, z) {
                if block.transparent() {
                    vertices.append(
                        &mut BlockFace::new(Direction::West, x as u32, y as u32, z as u32)
                            .vertices
                            .to_vec(),
                    );
                }
            } else {
                vertices.append(
                    &mut BlockFace::new(Direction::West, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            }

            if z == 15 {
                vertices.append(
                    &mut BlockFace::new(Direction::South, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            } else if let Some(block) = chunk.get_block(x, y, z + 1) {
                if block.transparent() {
                    vertices.append(
                        &mut BlockFace::new(Direction::South, x as u32, y as u32, z as u32)
                            .vertices
                            .to_vec(),
                    );
                }
            } else {
                vertices.append(
                    &mut BlockFace::new(Direction::South, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            }

            if z == 0 {
                vertices.append(
                    &mut BlockFace::new(Direction::North, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            } else if let Some(block) = chunk.get_block(x, y, z - 1) {
                if block.transparent() {
                    vertices.append(
                        &mut BlockFace::new(Direction::North, x as u32, y as u32, z as u32)
                            .vertices
                            .to_vec(),
                    );
                }
            } else {
                vertices.append(
                    &mut BlockFace::new(Direction::North, x as u32, y as u32, z as u32)
                        .vertices
                        .to_vec(),
                );
            }
        }

        Self {
            buffer: renderer.create_buffer(
                bytemuck::cast_slice(vertices.as_slice()),
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            ),
        }
    }
}

pub struct ChunkIndices {
    pub buffer: Index,
    pub len: usize,
}

impl ChunkIndices {
    pub fn new(renderer: &mut Renderer) -> Self {
        let mut indices = Vec::new();

        for face in 0..(16 * 16 * 16 * 6) as usize {
            indices.insert(face * 6 + 0, face as u32 * 4 + 2);
            indices.insert(face * 6 + 1, face as u32 * 4 + 1);
            indices.insert(face * 6 + 2, face as u32 * 4 + 0);
            indices.insert(face * 6 + 3, face as u32 * 4 + 0);
            indices.insert(face * 6 + 4, face as u32 * 4 + 3);
            indices.insert(face * 6 + 5, face as u32 * 4 + 2);
        }

        Self {
            buffer: renderer.create_buffer(
                bytemuck::cast_slice(indices.as_slice()),
                BufferUsages::INDEX | BufferUsages::COPY_DST,
            ),
            len: indices.len(),
        }
    }
}
