use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub base_url: String,
    pub token: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let config_file_dir = get_config_file_path().expect("Config directory not found");
        let config_content = std::fs::read_to_string(config_file_dir).expect("Run: cairos setup");
        let config = toml::from_str::<Config>(&config_content).expect("Config file is weird");

        Self {
            base_url: config.base_url,
            token: config.token,
        }
    }
}

pub fn get_config_file_path() -> Option<PathBuf> {
    let base = if cfg!(target_os = "linux") {
        env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .ok()
            .or_else(|| {
                env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
    } else if cfg!(target_os = "macos") {
        env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    } else if cfg!(target_os = "windows") {
        env::var("APPDATA").ok().map(PathBuf::from)
    } else {
        None
    }?;

    Some(base.join("cairos").join("config.toml"))
}
