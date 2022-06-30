use bevy_ecs::prelude::{ResMut, Schedule, SystemStage};
use bevy_ecs::schedule::Stage;
use bevy_ecs::system::Res;
use bevy_ecs::world::World;
use wgpu::{BufferUsages, IndexFormat, SurfaceError};
use winit::error::OsError;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use crate::client::assets::{AssetManager, Identifier};
use crate::client::chunk::ChunkMesh;
use crate::client::renderer::Renderer;
use crate::client::voxel::VoxelVertex;

pub struct Game;

impl Game {
    pub fn run() -> Result<(), OsError>{
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title("yave").build(&event_loop)?;

        let mut main_schedule = Schedule::default();

        main_schedule.add_stage("main_loop", SystemStage::parallel());
        main_schedule.add_stage("render_loop", SystemStage::parallel()
            .with_system(Game::render));

        let mut world = World::new();
        world.insert_resource(window);

        let renderer = Renderer::new(world.get_resource::<Window>().unwrap());

        world.insert_resource(renderer);

        let assets = AssetManager::new(world.get_resource::<Renderer>().unwrap());

        world.insert_resource(assets);

        event_loop.run(move |e, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            let window = world.get_resource::<Window>().unwrap();

            match e {
                Event::WindowEvent { window_id, event} => {

                    if window_id == window.id() {
                        match event {
                            WindowEvent::Resized(new_size) => {
                                let mut renderer = world.get_resource_mut::<Renderer>().unwrap();
                                renderer.surface_config.width = new_size.width;
                                renderer.surface_config.height = new_size.height;

                                renderer.surface.configure(&renderer.device, &renderer.surface_config);
                            }
                            WindowEvent::CloseRequested => {
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::KeyboardInput { .. } => {}
                            WindowEvent::MouseInput { .. } => {}
                            _ => ()
                        }
                    }
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    main_schedule.run(&mut world);
                }
                _ => ()
            }
        });
    }

    pub fn render(mut renderer: ResMut<Renderer>, assets: Res<AssetManager>) {
        let output = renderer.surface.get_current_texture();

        match output {
            Ok(output) => {
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Command Encoder") });

                let mesh = ChunkMesh::new(&mut renderer, vec![
                    VoxelVertex::new(0, 1, 0, 0, 0),
                    VoxelVertex::new(0, 0, 0, 0, 0),
                    VoxelVertex::new(1, 0, 0, 0, 0),
                ]);

                let indices = [0u16, 1, 2];

                let indices = renderer.create_buffer(bytemuck::cast_slice(&indices), BufferUsages::INDEX | BufferUsages::COPY_DST);

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.2,
                            b: 1.0,
                            a: 1.0
                        }), store: true }
                    }],
                    depth_stencil_attachment: None
                });

                let world_pipeline = assets.get_pipeline(Identifier::new("base", "world")).unwrap();

                render_pass.set_pipeline(&world_pipeline.render_pipeline);
                render_pass.set_index_buffer(renderer.arena[indices].slice(..), IndexFormat::Uint16);
                render_pass.set_vertex_buffer(0, renderer.arena[mesh.buffer].slice(..));
                render_pass.draw_indexed(0..3, 0, 0..1);

                drop(render_pass);

                renderer.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            Err(SurfaceError::Lost) => {
                renderer.surface.configure(&renderer.device, &renderer.surface_config);
            }
            _ => ()
        }
    }
}