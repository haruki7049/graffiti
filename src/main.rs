use clap::Parser;
use std::net::TcpStream;
use tracing::{debug, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, filter, fmt};
use tungstenite::{Message, WebSocket, connect, stream::MaybeTlsStream};
use url::Url;

use graffiti::{CLIArgs, Configuration};

#[tracing::instrument]
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

    for mut socket in sockets {
        socket.send(Message::Text("[\"REQ\", \"sub1\", {}]".into()))?;

        loop {
            let msg = socket.read().expect("Error reading message");
            debug!("Received: {:?}", msg);

            match msg {
                Message::Text(ref bytes) => {
                    info!("Received: {}", bytes);
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

#[tracing::instrument]
fn connect_to_server(
    urls: Vec<Url>,
) -> Result<Vec<WebSocket<MaybeTlsStream<TcpStream>>>, Box<dyn std::error::Error>> {
    let mut result: Vec<WebSocket<MaybeTlsStream<TcpStream>>> = Vec::new();

    for url in urls {
        let (socket, response) = connect(url.as_str())?;
        debug!("Received from {}: {:?}", url, response);
        debug!("Create a socket from {}", url);

        result.push(socket);
    }

    return Ok(result);
}
