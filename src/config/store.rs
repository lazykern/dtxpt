use std::path::Path;

use anyhow::Result;
use directories::ProjectDirs;

use super::model::GameConfig;

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("net", "dtxpt", "dtxpt")
}

pub fn game_config_path() -> std::path::PathBuf {
    project_dirs()
        .map(|dirs| dirs.config_dir().join("config.ron"))
        .unwrap_or_else(|| std::path::PathBuf::from("config.ron"))
}

pub fn library_cache_path() -> std::path::PathBuf {
    project_dirs()
        .map(|dirs| dirs.data_local_dir().join("library_cache.ron"))
        .unwrap_or_else(|| std::path::PathBuf::from("library_cache.ron"))
}

pub fn load_game_config() -> GameConfig {
    let path = game_config_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        match ron::de::from_str::<GameConfig>(&text) {
            Ok(config) => return config,
            Err(err) => eprintln!("failed to parse {}: {err}", path.display()),
        }
    }

    let config = GameConfig::default();
    if let Err(err) = save_game_config(&config) {
        eprintln!("failed to write default {}: {err}", path.display());
    }
    config
}

pub fn save_game_config(config: &GameConfig) -> Result<()> {
    let path = game_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = ron::ser::to_string_pretty(config, ron::ser::PrettyConfig::default())?;
    std::fs::write(path, text)?;
    Ok(())
}

pub fn default_chart_path() -> String {
    std::env::args().nth(1).unwrap_or_default()
}

pub fn initial_chart_path(config: &GameConfig) -> String {
    if let Some(cli) = std::env::args().nth(1) {
        return cli;
    }
    if !config.last_chart_path.is_empty() && Path::new(&config.last_chart_path).exists() {
        return config.last_chart_path.clone();
    }
    default_chart_path()
}
