use log::info;
use winit::error::OsError;

#[tokio::main]
async fn main() -> Result<(), OsError> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    info!("Game starting");

    let addr;

    if args.len() < 2 {
        tokio::spawn(async move { yave::server::game::Game::run().unwrap() });
        addr = String::from("localhost:25000");
    } else {
        addr = args.get(1).unwrap().clone();
    }

    yave::client::game::Game::run(addr).await?;

    Ok(())
}
