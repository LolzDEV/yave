use crate::assets::{AssetManager, Identifier};
use crate::client::renderer::Renderer;
use bevy_ecs::prelude::Component;
use cgmath::{Matrix4, SquareMatrix, Vector3};
use thunderdome::Index;
use wgpu::BufferUsages;

#[derive(Debug, Clone, Copy, Component)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub scale: f32,
    pub rotation: Vector3<f32>,
}

impl Transform {
    pub fn from_position(pos: impl Into<Vector3<f32>>) -> Self {
        let pos = pos.into();

        Self {
            position: pos,
            scale: 1.,
            rotation: (0., 0., 0.).into(),
        }
    }

    pub fn create_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    pub transformation: [[f32; 4]; 4],
}

impl TransformUniform {
    pub fn new() -> Self {
        Self {
            transformation: Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, transform: Transform) {
        self.transformation = transform.create_matrix().into();
    }
}

impl Default for TransformUniform {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Component)]
pub struct TransformBundle {
    pub transform: Transform,
    pub transform_uniform: TransformUniform,
    pub buffer: Index,
    pub bind_group: Index,
}

impl TransformBundle {
    pub fn new(
        pos: impl Into<Vector3<f32>>,
        renderer: &mut Renderer,
        assets: &AssetManager,
    ) -> Self {
        let transform = Transform::from_position(pos);

        let mut transform_uniform = TransformUniform::new();

        transform_uniform.update(transform);

        let transform_buffer = renderer.create_buffer(
            bytemuck::cast_slice(&[transform_uniform]),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let buffer = renderer.get_buffer(transform_buffer);

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("base:transform"),
                layout: assets
                    .get_bind_group_layout(Identifier::new("base", "transform"))
                    .unwrap(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        let bind_group = renderer.insert_bind_group(bind_group);

        Self {
            transform,
            transform_uniform,
            buffer: transform_buffer,
            bind_group,
        }
    }
}
