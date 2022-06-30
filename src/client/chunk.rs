use bevy_ecs::change_detection::ResMut;
use thunderdome::Index;
use wgpu::BufferUsages;
use crate::client::renderer::Renderer;
use crate::client::voxel::VoxelVertex;

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkMesh {
    pub buffer: Index,
}

impl ChunkMesh {
    pub fn new(renderer: &mut Renderer, vertices: Vec<VoxelVertex>) -> Self {
        Self {
            buffer: renderer.create_buffer(bytemuck::cast_slice(vertices.as_slice()), BufferUsages::VERTEX | BufferUsages::COPY_DST),
        }
    }
}