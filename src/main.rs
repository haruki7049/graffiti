use tokio::sync::mpsc;
use clap::Parser;
use std::net::TcpStream;
use tracing::{debug, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, filter, fmt};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

use graffiti::{CLIArgs, Configuration, connect_to_server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(true).compact())
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

    debug!("relays: {:?}", configuration.relays());

    let sockets: Vec<WebSocket<MaybeTlsStream<TcpStream>>> =
        connect_to_server(configuration.relays())?;

    info!("Connected to the server");
    debug!("sockets: {:?}", sockets);

    mainloop(sockets).await?;

    Ok(())
}

#[tracing::instrument(skip(sockets))]
async fn mainloop(
    sockets: Vec<WebSocket<MaybeTlsStream<TcpStream>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    const QUERY: &str = "[\"REQ\", \"sub1\", {\"kinds\":[1], \"limit\":0}]";

    for mut socket in sockets {
        let (tx, rx) = mpsc::channel(256);

        debug!("Socket: {:?}", socket);
        socket.send(Message::Text(QUERY.into()))?;

        tokio::spawn(websocket_reader(socket, tx));
        tokio::spawn(display_event(rx));
    }

    std::thread::yield_now();

    Ok(())
}

#[tracing::instrument(skip(socket, tx))]
async fn websocket_reader(mut socket: WebSocket<MaybeTlsStream<TcpStream>>, tx: mpsc::Sender<String>) {
    loop {
        let msg = socket.read().expect("Error reading message");

        match msg {
            Message::Text(ref bytes) => {
                let s = bytes.as_str().to_string();
                let _ = tx.send(s);
            }
            Message::Pong(_) => (),
            v => {
                warn!("Received unexpected format: {:?}", v);
            }
        }
    }
}

#[tracing::instrument(skip(rx))]
async fn display_event(mut rx: mpsc::Receiver<String>) {
    loop {
        let s = rx.recv().await;

        info!("Received: {:?}", s);
    }
}
