use log::{debug, info};
use winit::error::OsError;
use crate::game::Game;

mod game;
mod client;
pub mod world;

#[cfg(test)]
mod tests;

fn main() -> Result<(), OsError> {
    env_logger::init();

    info!("Game starting");

    Game::run()
}
