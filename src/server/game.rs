use crate::network::Packet;
use crate::server::{
    split_socket, ClientEvent, Connection, Player, PlayerName, Position, SocketSender,
};
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
    pub fn run() -> io::Result<()> {
        let world = Arc::new(Mutex::new(World::new()));

        world
            .lock()
            .unwrap()
            .insert_resource(Events::<ClientEvent>::default());

        let mut main_schedule = Schedule::default();

        main_schedule.add_stage(
            "events",
            SystemStage::parallel().with_system(Events::<ClientEvent>::update_system),
        );

        main_schedule.add_stage(
            "main_loop",
            SystemStage::parallel().with_system(Game::handle_packets),
        );

        info!("Starting server on port 25000");

        let socket = UdpSocket::bind("0.0.0.0:25000")?;

        let (sender, mut receiver) = split_socket(socket);

        world.lock().unwrap().insert_resource(sender);

        let world_clone = world.clone();

        thread::spawn(|| {
            block_on(async move {
                let world = world_clone;

                loop {
                    let mut data = vec![0u8; std::mem::size_of::<Packet>()];
                    if let Ok((size, peer)) = receiver.recv_from(&mut data).await {
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

        loop {
            main_schedule.run(&mut world.lock().unwrap());

            // Run server loop 20 times a second.
            while Instant::now() - last_time < Duration::from_secs_f64(1. / 20.) {
                continue;
            }
            last_time = Instant::now();
        }
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
                    for (player, _position, connection) in players.iter() {
                        block_on(sender.send_to(
                            event.packet.clone().encode().unwrap().as_slice(),
                            &connection.peer,
                        ))
                        .unwrap();

                        block_on(
                            sender.send_to(
                                Packet::Connection {
                                    user: player.name.name.clone(),
                                }
                                .encode()
                                .unwrap()
                                .as_slice(),
                                &event.peer,
                            ),
                        )
                        .unwrap();
                    }

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
                            block_on(
                                sender.send_to(
                                    Packet::PlayerPosition {
                                        x: position.x,
                                        y: position.y,
                                        z: position.z,
                                        name: name.clone(),
                                    }
                                    .encode()
                                    .unwrap()
                                    .as_slice(),
                                    &event.peer,
                                ),
                            )
                            .unwrap();
                        }
                    }
                }
                _ => (),
            }
        }
    }
}