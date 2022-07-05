use log::info;
use winit::error::OsError;

#[tokio::main]
async fn main() -> Result<(), OsError> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    let mut addr = String::from("localhost:25000");
    let mut username = String::from("singleplayer");
    let mut dedicated = false;
    let mut port = String::from("25000");

    for (i, arg) in args.iter().enumerate() {
        if arg == "--connect" {
            addr = args.get(i + 1).unwrap().clone();
        }

        if arg == "--username" {
            username = args.get(i + 1).unwrap().clone();
        }

        if arg == "--dedicated" {
            dedicated = true;
        }

        if arg == "--port" {
            port = args.get(i + 1).unwrap().clone();
        }
    }

    info!("Game starting");

    if !dedicated {
        if args.len() < 2 {
            tokio::spawn(async move { yave::server::game::Game::run(port).unwrap() });
        } else {
            addr = args.get(1).unwrap().clone();
        }

        yave::client::game::Game::run(addr, username).await?;
    } else {
        yave::server::game::Game::run(port).unwrap()
    }

    Ok(())
}
