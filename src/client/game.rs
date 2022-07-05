use crate::assets::{AssetManager, Identifier};
use crate::client::camera::{Camera, CameraBundle, CameraController, CameraUniform, Projection};
use crate::client::chunk::ChunkMesh;
use crate::client::player::{Player, PlayerController};
use crate::client::renderer::Renderer;
use crate::client::transform::TransformBundle;
use crate::client::voxel::VoxelVertex;
use crate::client::ServerEvent;
use crate::network::Packet;
use crate::server::{split_socket, SocketSender};
use crate::world::chunk::Chunk;
use crate::{DeltaTime, KeyboardEvent, MouseMotion};
use bevy_ecs::event::{EventReader, Events};
use bevy_ecs::prelude::{Commands, Query, ResMut, Schedule, SystemStage};
use bevy_ecs::schedule::Stage;
use bevy_ecs::system::Res;
use bevy_ecs::world::World;
use cgmath::Deg;
use log::info;
use pollster::block_on;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wgpu::{BufferUsages, IndexFormat, SurfaceError};
use winit::error::OsError;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct Game;

impl Game {
    pub async fn run(addr: String) -> Result<(), OsError> {
        Chunk::new(0, 0);

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title("yave").build(&event_loop)?;

        let mut main_schedule = Schedule::default();

        let mut setup_schedule = Schedule::default();

        let world = Arc::new(Mutex::new(World::new()));

        world
            .lock()
            .unwrap()
            .insert_resource(Events::<KeyboardEvent>::default());
        world
            .lock()
            .unwrap()
            .insert_resource(Events::<MouseMotion>::default());

        world
            .lock()
            .unwrap()
            .insert_resource(Events::<ServerEvent>::default());

        setup_schedule.add_stage("setup", SystemStage::parallel().with_system(Game::setup));
        main_schedule.add_stage(
            "events",
            SystemStage::parallel()
                .with_system(Events::<KeyboardEvent>::update_system)
                .with_system(Events::<MouseMotion>::update_system)
                .with_system(Events::<ServerEvent>::update_system),
        );
        main_schedule.add_stage(
            "main_loop",
            SystemStage::parallel()
                .with_system(Game::update)
                .with_system(Game::handle_keyboard)
                .with_system(Game::handle_mouse)
                .with_system(Game::handle_packets),
        );
        main_schedule.add_stage(
            "render_loop",
            SystemStage::parallel().with_system(Game::render),
        );

        world.lock().unwrap().insert_resource(window);

        let renderer = Renderer::new(world.lock().unwrap().get_resource::<Window>().unwrap());

        world.lock().unwrap().insert_resource(renderer);

        let assets = AssetManager::new(world.lock().unwrap().get_resource::<Renderer>().unwrap());

        world.lock().unwrap().insert_resource(assets);

        world
            .lock()
            .unwrap()
            .insert_resource(DeltaTime(Duration::from_secs_f32(0.0)));

        setup_schedule.run(&mut world.lock().unwrap());

        let mut last_frame_time = Instant::now();

        let world_clone = world.clone();

        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

        socket.connect(addr).unwrap();

        let (mut sender, mut receiver) = split_socket(socket);

        info!("Connecting");

        sender
            .send(
                Packet::Connection {
                    user: "Lolz".to_string(),
                }
                .encode()
                .unwrap()
                .as_slice(),
            )
            .await
            .unwrap();

        world.lock().unwrap().insert_resource(sender);

        // Spawn network thread to listen for packets.
        tokio::spawn(async move {
            let world = world_clone;

            loop {
                let mut data = vec![0u8; std::mem::size_of::<Packet>()];
                if let Ok((size, _peer)) = receiver.recv_from(&mut data).await {
                    data.resize(size, 0);
                    if let Ok(packet) = Packet::decode(data) {
                        let mut world = world.lock().unwrap();
                        let mut client_events =
                            world.get_resource_mut::<Events<ServerEvent>>().unwrap();
                        client_events.send(ServerEvent { packet });
                    }
                }
            }
        });

        event_loop.run(move |e, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let world = world.clone();

            match e {
                Event::WindowEvent { window_id, event } => {
                    let mut world = world.lock().unwrap();
                    if window_id == world.get_resource::<Window>().unwrap().id() {
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
                                let window = world.get_resource_mut::<Window>().unwrap();
                                window.set_cursor_grab(is);
                                window.set_cursor_visible(!is);
                            }
                            _ => (),
                        }
                    }
                }
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        let mut world = world.lock().unwrap();
                        let mut mouse_events =
                            world.get_resource_mut::<Events<MouseMotion>>().unwrap();
                        mouse_events.send(MouseMotion { delta });
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    world
                        .lock()
                        .unwrap()
                        .get_resource::<Window>()
                        .unwrap()
                        .request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let delta_time = Instant::now() - last_frame_time;
                    last_frame_time = Instant::now();
                    world.lock().unwrap().insert_resource(DeltaTime(delta_time));
                    main_schedule.run(&mut world.lock().unwrap());
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

        commands.insert_resource(PlayerController::new(2., 0.5));
    }

    pub fn update(
        delta_time: Res<DeltaTime>,
        mut camera_bundle: ResMut<CameraBundle>,
        mut camera_controller: ResMut<CameraController>,
        renderer: Res<Renderer>,
        mut sender: ResMut<SocketSender>,
        mut player_controller: ResMut<PlayerController>,
        mut players: Query<(&Player, &mut TransformBundle)>,
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

        block_on(
            sender.send(
                Packet::Movement {
                    delta_x: camera_bundle.camera.position.x as f64,
                    delta_y: camera_bundle.camera.position.y as f64,
                    delta_z: camera_bundle.camera.position.z as f64,
                }
                .encode()
                .unwrap()
                .as_slice(),
            ),
        )
        .unwrap();

        *player_controller = PlayerController::new(2., 0.5);

        for (player, mut transform_bundle) in players.iter_mut() {
            block_on(
                sender.send(
                    Packet::PositionRequest {
                        name: player.name.clone(),
                    }
                    .encode()
                    .unwrap()
                    .as_slice(),
                ),
            )
            .unwrap();

            let transform = transform_bundle.transform;

            transform_bundle.transform_uniform.update(transform);
            renderer.queue.write_buffer(
                renderer.get_buffer(transform_bundle.buffer),
                0,
                bytemuck::cast_slice(&[transform_bundle.transform_uniform]),
            );
        }
    }

    pub fn handle_packets(
        mut commands: Commands,
        mut events: EventReader<ServerEvent>,
        mut renderer: ResMut<Renderer>,
        assets: Res<AssetManager>,
        mut players: Query<(&Player, &mut TransformBundle)>,
    ) {
        for event in events.iter() {
            match &event.packet {
                Packet::OnlinePlayers { players } => {
                    for player in players {
                        commands
                            .spawn()
                            .insert(Player {
                                name: player.name.clone(),
                            })
                            .insert(TransformBundle::new(
                                (player.x, player.y, player.z),
                                &mut renderer,
                                &assets,
                            ));
                    }
                }
                Packet::PlayerPosition { x, y, z, name } => {
                    for (player, mut transform_bundle) in players.iter_mut() {
                        if player.name == name.clone() {
                            transform_bundle.transform.position =
                                (*x as f32, *y as f32, *z as f32).into();
                        }
                    }
                }
                _ => (),
            }
        }
    }

    pub fn handle_keyboard(
        mut events: EventReader<KeyboardEvent>,
        mut camera_controller: ResMut<CameraController>,
        mut player_controller: ResMut<PlayerController>,
    ) {
        for event in events.iter() {
            if let KeyboardInput {
                virtual_keycode: Some(keycode),
                state,
                ..
            } = event.input
            {
                camera_controller.process_keyboard(keycode, state);
                player_controller.process_keyboard(keycode, state);
            }
        }
    }

    pub fn handle_mouse(
        mut events: EventReader<MouseMotion>,
        mut camera_controller: ResMut<CameraController>,
        mut player_controller: ResMut<PlayerController>,
    ) {
        for event in events.iter() {
            //camera_controller.process_mouse(event.delta.0, event.delta.1);
            player_controller.process_mouse(event.delta.0, event.delta.1);
        }
    }

    pub fn render(
        mut renderer: ResMut<Renderer>,
        assets: Res<AssetManager>,
        camera_bundle: Res<CameraBundle>,
        players: Query<(&Player, &TransformBundle)>,
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

                for (_player, transform_bundle) in players.iter() {
                    render_pass.set_bind_group(
                        1,
                        renderer.get_bind_group(transform_bundle.bind_group),
                        &[],
                    );
                    render_pass.draw_indexed(0..3, 0, 0..1);
                }

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
