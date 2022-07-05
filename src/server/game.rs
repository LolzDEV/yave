use crate::network::{split_socket, OnlinePlayer, Packet, SocketSender};
use crate::server::{ClientEvent, Connection, Player, PlayerName, Position};
use crate::world::chunk::Chunk;
use crate::world::Chunks;
use bevy_ecs::event::Events;
use bevy_ecs::prelude::{Commands, EventReader, Query, Schedule, SystemStage, World};
use bevy_ecs::schedule::Stage;
use bevy_ecs::system::ResMut;
use log::info;
use pollster::block_on;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{io, thread};

pub struct Game;

impl Game {
    pub fn run(port: String) -> io::Result<()> {
        let world = Arc::new(Mutex::new(World::new()));

        world
            .lock()
            .unwrap()
            .insert_resource(Events::<ClientEvent>::default());

        let mut setup_schedule = Schedule::default();

        setup_schedule.add_stage("setup", SystemStage::parallel().with_system(Game::setup));

        let mut main_schedule = Schedule::default();

        main_schedule.add_stage(
            "events",
            SystemStage::parallel().with_system(Events::<ClientEvent>::update_system),
        );

        main_schedule.add_stage(
            "main_loop",
            SystemStage::parallel()
                .with_system(Game::handle_packets)
                .with_system(Game::update_chunks),
        );

        info!("Starting server on port 25000");

        let socket = UdpSocket::bind(format!("0.0.0.0:{port}"))?;

        let (sender, mut receiver) = split_socket(socket);

        world.lock().unwrap().insert_resource(sender);

        let world_clone = world.clone();

        thread::spawn(|| {
            block_on(async move {
                let world = world_clone;

                loop {
                    let mut data = vec![0u8; std::mem::size_of::<Packet>()];
                    if let Ok((size, peer)) = receiver.recv_from(&mut data) {
                        data.resize(size, 0);
                        if let Ok(packet) = Packet::decode(data) {
                            let mut world = world.lock().unwrap();
                            let mut client_events =
                                world.get_resource_mut::<Events<ClientEvent>>().unwrap();
                            client_events.send(ClientEvent { packet, peer });
                        }
                    }
                }
            })
        });

        let mut last_time = Instant::now();

        setup_schedule.run(&mut world.lock().unwrap());

        loop {
            main_schedule.run(&mut world.lock().unwrap());

            // Run server loop 20 times a second.
            while Instant::now() - last_time < Duration::from_secs_f64(1. / 20.) {
                continue;
            }
            last_time = Instant::now();
        }
    }

    pub fn setup(mut commands: Commands) {
        commands.insert_resource(Chunks::default());
    }

    pub fn handle_packets(
        mut commands: Commands,
        mut events: EventReader<ClientEvent>,
        mut players: Query<(&Player, &mut Position, &Connection)>,
        mut sender: ResMut<SocketSender>,
    ) {
        for event in events.iter() {
            match &event.packet {
                Packet::Connection { user } => {
                    let mut online_players = Vec::new();
                    for (player, position, connection) in players.iter() {
                        sender
                            .send_to(event.packet.clone(), &connection.peer)
                            .unwrap();

                        online_players.push(OnlinePlayer {
                            name: player.name.name.clone(),
                            x: position.x as f32,
                            y: position.y as f32,
                            z: position.z as f32,
                        });
                    }

                    sender
                        .send_to(
                            Packet::OnlinePlayers {
                                players: online_players,
                            },
                            &event.peer,
                        )
                        .unwrap();

                    commands
                        .spawn()
                        .insert(Player {
                            name: PlayerName { name: user.clone() },
                        })
                        .insert(Position {
                            x: 0.,
                            y: 0.,
                            z: 10.,
                        })
                        .insert(Connection { peer: event.peer });

                    info!("Player {user} connected.");
                }
                Packet::Movement {
                    delta_x,
                    delta_y,
                    delta_z,
                } => {
                    for (_player, mut position, connection) in players.iter_mut() {
                        if connection.peer == event.peer {
                            position.x = *delta_x;
                            position.y = *delta_y;
                            position.z = *delta_z;
                        }
                    }
                }
                Packet::PositionRequest { name } => {
                    for (player, position, _connection) in players.iter_mut() {
                        if player.name.name == name.clone() {
                            sender
                                .send_to(
                                    Packet::PlayerPosition {
                                        x: position.x,
                                        y: position.y,
                                        z: position.z,
                                        name: name.clone(),
                                    },
                                    &event.peer,
                                )
                                .unwrap();
                        }
                    }
                }
                _ => (),
            }
        }
    }

    pub fn update_chunks(
        mut chunks: ResMut<Chunks>,
        players: Query<(&Player, &Position, &Connection)>,
        mut sender: ResMut<SocketSender>,
    ) {
        let mut to_unload = vec![];

        for (i, chunk) in chunks.chunks.iter().enumerate() {
            let mut unload = true;
            for (_player, position, _connection) in players.iter() {
                if position.x as i64 / 16 == chunk.x || position.z as i64 / 16 == chunk.y {
                    unload = false;
                }

                if unload {
                    to_unload.push(i);
                }
            }
        }

        let mut connections = Vec::new();

        for (_player, _position, connection) in players.iter() {
            connections.push(connection);
        }

        for i in to_unload {
            let chunk = chunks.chunks.get(i).unwrap();

            for connection in connections.iter() {
                sender
                    .send_to(
                        Packet::UnloadChunk {
                            x: chunk.x,
                            y: chunk.y,
                        },
                        &connection.peer,
                    )
                    .unwrap();
            }

            chunks.chunks.remove(i);
        }

        for (_player, position, _connection) in players.iter() {
            if let None = chunks.get_chunk(position.x as i64 / 16, position.z as i64 / 16) {
                info!("Generating chunk.");
                let chunk = Chunk::new(position.x as i64 / 16, position.z as i64 / 16);

                for connection in connections.iter() {
                    sender
                        .send_to(
                            Packet::Chunk {
                                x: chunk.x,
                                y: chunk.y,
                                groups: chunk.compress(),
                            },
                            &connection.peer,
                        )
                        .unwrap();
                }
                info!("Done!");

                chunks.chunks.push(chunk);
            }
        }
    }
}
