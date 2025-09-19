use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io,
    path::PathBuf,
};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub base_url: String,
    pub token: Option<String>,
}

impl Config {
    pub fn load() -> Self {
        let config_file_path = get_config_file_path().expect("Error on get config file path");
        let config_content = get_or_create_config_content(config_file_path)
            .expect("Error on get config file content");
        let config = toml::from_str::<Config>(&config_content).expect("Config file is weird");

        Self {
            base_url: config.base_url,
            token: config.token,
        }
    }
}

pub fn get_or_create_config_content(config_file_path: PathBuf) -> io::Result<String> {
    if !config_file_path.exists() {
        config_file_path
            .parent()
            .map(|parent| fs::create_dir_all(parent));

        File::create(&config_file_path)?;

        let my_config = Config {
            base_url: "https://localhost".to_owned(),
            token: None,
        };

        let toml_content = toml::to_string_pretty(&my_config).expect("Toml serialization failed");

        fs::write(&config_file_path, toml_content)?;
    }

    std::fs::read_to_string(config_file_path)
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
