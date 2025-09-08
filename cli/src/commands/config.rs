use crate::config::{Config, get_config_file_path};
use anyhow::Context;
use std::fs;

pub fn setup(base_url: String) -> anyhow::Result<()> {
    let config_file = get_config_file_path().context("Config directory not found")?;

    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent).context("failed to create config directory")?;
    }

    let default = Config {
        base_url,
        token: None,
    };

    let toml_str =
        toml::to_string_pretty(&default).context("failed to serialize default config")?;
    fs::write(&config_file, toml_str).context("failed to write default config")?;

    Ok(())
}

pub fn set_token(token: String) -> anyhow::Result<()> {
    let config_file = get_config_file_path().context("Config file not found")?;
    let mut config: Config = toml::from_str(&fs::read_to_string(&config_file)?)
        .context("failed to parse config file")?;

    config.token = Some(token);

    let toml_str = toml::to_string_pretty(&config).context("failed to serialize config")?;
    fs::write(&config_file, toml_str).context("failed to write config file")?;

    Ok(())
}
