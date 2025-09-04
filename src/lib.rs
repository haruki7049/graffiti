use clap::Parser;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::{
    path::PathBuf,
    sync::{LazyLock, Mutex},
};
use tracing::{debug, warn};
use tungstenite::{WebSocket, stream::MaybeTlsStream};
use url::Url;

#[derive(Parser)]
#[clap(about, version, author)]
pub struct CLIArgs {
    #[arg(short, long, default_value = DEFAULT_CONFIG_PATH.lock().unwrap().display().to_string())]
    pub config_file: PathBuf,
}

pub static DEFAULT_CONFIG_PATH: LazyLock<Mutex<PathBuf>> = LazyLock::new(|| {
    let proj_dirs = ProjectDirs::from("dev", "haruki7049", "graffiti")
        .expect("Failed to search ProjectDirs for dev.haruki7049.spacerobo");
    let mut config_path: PathBuf = proj_dirs.config_dir().to_path_buf();
    let filename: &str = "config.toml";

    config_path.push(filename);
    Mutex::new(config_path)
});

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Configuration {
    relays: Vec<Url>,
}

impl Configuration {
    pub fn relays(&self) -> Vec<Url> {
        self.relays.clone()
    }
}

#[tracing::instrument]
pub fn connect_to_server(
    urls: Vec<Url>,
) -> Result<Vec<WebSocket<MaybeTlsStream<TcpStream>>>, Box<dyn std::error::Error>> {
    let mut result: Vec<WebSocket<MaybeTlsStream<TcpStream>>> = Vec::new();

    for url in urls {
        let (socket, response) = tungstenite::connect(url.as_str())?;
        debug!("Received from {}: {:?}", url, response);
        debug!("Create a socket from {}", url);

        result.push(socket);
    }

    return Ok(result);
}
