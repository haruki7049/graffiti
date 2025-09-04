use clap::Parser;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{LazyLock, Mutex},
};
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
