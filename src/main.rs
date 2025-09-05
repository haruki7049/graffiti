use tokio::sync::mpsc;
use clap::Parser;
use tokio::net::TcpStream;
use tracing::{debug, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, filter, fmt};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

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

    let sockets: Vec<WebSocketStream<MaybeTlsStream<TcpStream>>> =
        connect_to_server(configuration.relays()).await?;

    info!("Connected to the server");
    debug!("sockets: {:?}", sockets);

    mainloop(sockets).await?;

    Ok(())
}

#[tracing::instrument]
async fn mainloop(
    sockets: Vec<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    const QUERY: &str = "[\"REQ\", \"sub1\", {\"kinds\":[1], \"limit\":0}]";

    for mut socket in sockets {
        let (tx, rx) = mpsc::channel(256);

        debug!("Socket: {:?}", socket);
        socket.send(Message::Text(QUERY.into())).await?;

        let reader = tokio::task::spawn(websocket_reader(socket, tx));
        let display = tokio::task::spawn(display_event(rx));

        let _ = tokio::join!(reader, display);
    }

    Ok(())
}

#[tracing::instrument(skip(socket, tx))]
async fn websocket_reader(
    mut socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    tx: mpsc::Sender<String>,
) {
    while let Some(msg) = socket.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let t: String = text.to_string();

                if let Err(e) = tx.send(t).await {
                    warn!("Failed to send message to channel: {:?}", e);
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                debug!("Received Pong");
            }
            Ok(other) => {
                warn!("Received unexpected message: {:?}", other);
            }
            Err(e) => {
                warn!("Error reading message: {:?}", e);
                break;
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
