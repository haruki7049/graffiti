use serde::{Serialize, Deserialize};
use std::{
    path::PathBuf,
    sync::{Mutex, LazyLock},
};
use directories::ProjectDirs;
use clap::Parser;
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

    let args: CLIArgs = CLIArgs::parse();
    let configuration: Configuration = confy::load_path(&args.config_file)?;

    if configuration.relays.is_empty() {
        panic!("No relays found.");
    }

    let sockets: Vec<WebSocket<MaybeTlsStream<TcpStream>>> = connect_to_server(configuration.relays)?;

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

#[derive(Parser)]
#[clap(about, version, author)]
struct CLIArgs {
    #[arg(short, long, default_value = DEFAULT_CONFIG_PATH.lock().unwrap().display().to_string())]
    config_file: PathBuf,
}

static DEFAULT_CONFIG_PATH: LazyLock<Mutex<PathBuf>> = LazyLock::new(|| {
    let proj_dirs = ProjectDirs::from("dev", "haruki7049", "graffiti")
        .expect("Failed to search ProjectDirs for dev.haruki7049.spacerobo");
    let mut config_path: PathBuf = proj_dirs.config_dir().to_path_buf();
    let filename: &str = "config.toml";

    config_path.push(filename);
    Mutex::new(config_path)
});

#[derive(Debug, Serialize, Deserialize, Default)]
struct Configuration {
    relays: Vec<Url>,
}
