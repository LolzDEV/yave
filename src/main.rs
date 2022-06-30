use log::{info};
use winit::error::OsError;
use yave::game::Game;

fn main() -> Result<(), OsError> {
    env_logger::init();

    info!("Game starting");

    Game::run()
}
