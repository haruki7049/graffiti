use clap::Parser;
use std::net::TcpStream;
use tracing::{debug, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, filter, fmt};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

use graffiti::{CLIArgs, Configuration, connect_to_server};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args: CLIArgs = CLIArgs::parse();
    let configuration: Configuration = confy::load_path(&args.config_file)?;

    if configuration.relays().is_empty() {
        panic!("No relays found.");
    }

    let sockets: Vec<WebSocket<MaybeTlsStream<TcpStream>>> =
        connect_to_server(configuration.relays())?;

    info!("Connected to the server");
    debug!("sockets: {:?}", sockets);

    mainloop(sockets)?;

    Ok(())
}

fn mainloop(
    sockets: Vec<WebSocket<MaybeTlsStream<TcpStream>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    const QUERY: &str = "[\"REQ\", \"sub1\", {\"kinds\":[1], \"limit\":0}]";

    for mut socket in sockets {
        socket.send(Message::Text(QUERY.into()))?;

        loop {
            let msg = socket.read().expect("Error reading message");
            debug!("Received: {:?}", msg);

            match msg {
                Message::Text(ref bytes) => {
                    debug!("Received: {}", bytes);
                }
                Message::Pong(_) => (),
                v => {
                    warn!("Received unexpected format: {:?}", v);
                }
            }
        }
    }

    loop {}
}
