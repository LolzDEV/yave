use crate::client::assets::{AssetManager, Identifier};
use crate::client::camera::{Camera, CameraBundle, CameraController, CameraUniform, Projection};
use crate::client::chunk::ChunkMesh;
use crate::client::renderer::Renderer;
use crate::client::voxel::VoxelVertex;
use crate::{DeltaTime, KeyboardEvent, MouseMotion};
use bevy_ecs::event::{EventReader, Events};
use bevy_ecs::prelude::{Commands, ResMut, Schedule, SystemStage};
use bevy_ecs::schedule::Stage;
use bevy_ecs::system::Res;
use bevy_ecs::world::World;
use cgmath::Deg;
use std::time::{Duration, Instant};
use wgpu::{BufferUsages, IndexFormat, SurfaceError};
use winit::error::OsError;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct Game;

impl Game {
    pub fn run() -> Result<(), OsError> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title("yave").build(&event_loop)?;

        let mut main_schedule = Schedule::default();

        let mut setup_schedule = Schedule::default();

        let mut world = World::new();

        world.insert_resource(Events::<KeyboardEvent>::default());
        world.insert_resource(Events::<MouseMotion>::default());

        setup_schedule.add_stage("setup", SystemStage::parallel().with_system(Game::setup));
        main_schedule.add_stage(
            "events",
            SystemStage::parallel()
                .with_system(Events::<KeyboardEvent>::update_system)
                .with_system(Events::<MouseMotion>::update_system),
        );
        main_schedule.add_stage(
            "main_loop",
            SystemStage::parallel()
                .with_system(Game::update)
                .with_system(Game::handle_keyboard)
                .with_system(Game::handle_mouse),
        );
        main_schedule.add_stage(
            "render_loop",
            SystemStage::parallel().with_system(Game::render),
        );

        world.insert_resource(window);

        let renderer = Renderer::new(world.get_resource::<Window>().unwrap());

        world.insert_resource(renderer);

        let assets = AssetManager::new(world.get_resource::<Renderer>().unwrap());

        world.insert_resource(assets);

        world.insert_resource(DeltaTime(Duration::from_secs_f32(0.0)));

        setup_schedule.run(&mut world);

        let mut last_frame_time = Instant::now();

        event_loop.run(move |e, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            let window = world.get_resource::<Window>().unwrap();

            match e {
                Event::WindowEvent { window_id, event } => {
                    if window_id == window.id() {
                        match event {
                            WindowEvent::Resized(new_size) => {
                                let mut renderer = world.get_resource_mut::<Renderer>().unwrap();
                                renderer.surface_config.width = new_size.width;
                                renderer.surface_config.height = new_size.height;

                                renderer
                                    .surface
                                    .configure(&renderer.device, &renderer.surface_config);
                            }
                            WindowEvent::CloseRequested => {
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::KeyboardInput { input, .. } => {
                                let mut window_events =
                                    world.get_resource_mut::<Events<KeyboardEvent>>().unwrap();
                                window_events.send(KeyboardEvent { input });
                                if let KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    state: ElementState::Released,
                                    ..
                                } = input
                                {
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                            WindowEvent::Focused(is) => {
                                window.set_cursor_grab(is).expect("Failed to grub cursor");
                                window.set_cursor_visible(!is);
                            }
                            _ => (),
                        }
                    }
                }
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        let mut mouse_events =
                            world.get_resource_mut::<Events<MouseMotion>>().unwrap();
                        mouse_events.send(MouseMotion { delta });
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let delta_time = Instant::now() - last_frame_time;
                    last_frame_time = Instant::now();
                    world.insert_resource(DeltaTime(delta_time));
                    main_schedule.run(&mut world);
                }
                _ => (),
            }
        });
    }

    pub fn setup(
        mut commands: Commands,
        mut renderer: ResMut<Renderer>,
        assets: Res<AssetManager>,
        window: Res<Window>,
    ) {
        let camera = Camera::new((0., 0., 10.), Deg(-90.), Deg(-20.));
        let projection = Projection::new(
            window.inner_size().width as f32,
            window.inner_size().height as f32,
            Deg(45.),
            0.1,
            100.,
        );

        let mut camera_uniform = CameraUniform::new();

        camera_uniform.update(camera, projection);

        let camera_buffer = renderer.create_buffer(
            bytemuck::cast_slice(&[camera_uniform]),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let buffer = renderer.get_buffer(camera_buffer);

        let bind_group = renderer
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("base:camera"),
                layout: assets
                    .get_bind_group_layout(Identifier::new("base", "camera"))
                    .unwrap(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        let bind_group = renderer.insert_bind_group(bind_group);

        let camera_bundle = CameraBundle {
            camera,
            camera_uniform,
            buffer: camera_buffer,
            bind_group,
            projection,
        };

        commands.insert_resource(camera_bundle);
        commands.insert_resource(CameraController::new(2., 0.5));
    }

    pub fn update(
        delta_time: Res<DeltaTime>,
        mut camera_bundle: ResMut<CameraBundle>,
        mut camera_controller: ResMut<CameraController>,
        renderer: Res<Renderer>,
    ) {
        camera_controller.update_camera(&mut camera_bundle.camera, delta_time.0.as_secs_f32());

        let bundle_clone = *camera_bundle;

        camera_bundle
            .camera_uniform
            .update(bundle_clone.camera, bundle_clone.projection);

        renderer.queue.write_buffer(
            renderer.get_buffer(camera_bundle.buffer),
            0,
            bytemuck::cast_slice(&[camera_bundle.camera_uniform]),
        );
    }

    pub fn handle_keyboard(
        mut events: EventReader<KeyboardEvent>,
        mut camera_controller: ResMut<CameraController>,
    ) {
        for event in events.iter() {
            if let KeyboardInput {
                virtual_keycode: Some(keycode),
                state,
                ..
            } = event.input
            {
                camera_controller.process_keyboard(keycode, state);
            }
        }
    }

    pub fn handle_mouse(
        mut events: EventReader<MouseMotion>,
        mut camera_controller: ResMut<CameraController>,
    ) {
        for event in events.iter() {
            camera_controller.process_mouse(event.delta.0, event.delta.1);
        }
    }

    pub fn render(
        mut renderer: ResMut<Renderer>,
        assets: Res<AssetManager>,
        camera_bundle: Res<CameraBundle>,
    ) {
        let output = renderer.surface.get_current_texture();

        match output {
            Ok(output) => {
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    renderer
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Command Encoder"),
                        });

                let mesh = ChunkMesh::new(
                    &mut renderer,
                    vec![
                        VoxelVertex::new(0, 1, 0, 0, 0),
                        VoxelVertex::new(0, 0, 0, 0, 0),
                        VoxelVertex::new(1, 0, 0, 0, 0),
                    ],
                );

                let indices = [0u16, 1, 2];

                let indices = renderer.create_buffer(
                    bytemuck::cast_slice(&indices),
                    BufferUsages::INDEX | BufferUsages::COPY_DST,
                );

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.2,
                                g: 0.2,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                let world_pipeline = assets
                    .get_pipeline(Identifier::new("base", "world"))
                    .unwrap();

                render_pass.set_pipeline(&world_pipeline.render_pipeline);
                render_pass.set_bind_group(
                    0,
                    renderer.get_bind_group(camera_bundle.bind_group),
                    &[],
                );
                render_pass
                    .set_index_buffer(renderer.buffers[indices].slice(..), IndexFormat::Uint16);
                render_pass.set_vertex_buffer(0, renderer.buffers[mesh.buffer].slice(..));
                render_pass.draw_indexed(0..3, 0, 0..1);

                drop(render_pass);

                renderer.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            Err(SurfaceError::Lost) => {
                renderer
                    .surface
                    .configure(&renderer.device, &renderer.surface_config);
            }
            _ => (),
        }
    }
}
