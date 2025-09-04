use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};
use std::net::TcpStream;
use tungstenite::{WebSocket, connect, Message, stream::MaybeTlsStream};
use tracing::{debug, info};
use url::Url;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let urls: Vec<Url> = vec![
        Url::parse("wss://yabu.me")?,
        Url::parse("wss://yabu.me")?,
    ];

    let sockets: Vec<WebSocket<MaybeTlsStream<TcpStream>>> = connect_to_server(urls)?;

    info!("Connected to the server");
    debug!("sockets: {:?}", sockets);

    for mut socket in sockets {
        socket.send(Message::Text("[\"REQ\", \"sub1\", {}]".into())).unwrap();
        loop {
            let msg = socket.read().expect("Error reading message");
            debug!("Received: {msg}");
        }
    }

    loop {}

    Ok(())
}

#[tracing::instrument]
fn connect_to_server(urls: Vec<Url>) -> Result<Vec<WebSocket<MaybeTlsStream<TcpStream>>>, Box<dyn std::error::Error>> {
    let mut result: Vec<WebSocket<MaybeTlsStream<TcpStream>>> = Vec::new();

    for url in urls {
        let (socket, response) = connect(url.as_str())?;
        debug!("Received from {}: {:?}", url, response);
        debug!("Create a socket from {}", url);

        result.push(socket);
    }

    return Ok(result);
}
