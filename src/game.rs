use bevy_ecs::prelude::{Schedule, SystemStage};
use bevy_ecs::schedule::Stage;
use bevy_ecs::world::World;
use winit::error::OsError;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use crate::client::assets::AssetManager;
use crate::client::renderer::Renderer;

pub struct Game;

impl Game {
    pub fn run() -> Result<(), OsError>{
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title("yave").build(&event_loop)?;

        let mut schedule = Schedule::default();

        schedule.add_stage("main_loop", SystemStage::parallel());

        let mut world = World::new();
        world.insert_resource(window);

        let renderer = Renderer::new(world.get_resource::<Window>().unwrap());

        world.insert_resource(renderer);

        let assets = AssetManager::new(world.get_resource::<Renderer>().unwrap());

        world.insert_resource(assets);

        event_loop.run(move |e, _, control_flow| {
            schedule.run(&mut world);
            *control_flow = ControlFlow::Poll;

            match e {
                Event::WindowEvent { window_id, event} => {
                    let window = world.get_resource::<Window>().unwrap();

                    if window_id == window.id() {
                        match event {
                            WindowEvent::Resized(new_size) => {}
                            WindowEvent::CloseRequested => {
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::KeyboardInput { .. } => {}
                            WindowEvent::MouseInput { .. } => {}
                            _ => ()
                        }
                    }
                }
                Event::MainEventsCleared => {}
                Event::RedrawRequested(_) => {}
                _ => ()
            }
        });
    }
}