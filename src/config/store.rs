use std::path::Path;

use anyhow::Result;
use directories::ProjectDirs;

use super::model::{FpsCap, GameConfig};

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

/// Migrate a v9 config (had `vsync: bool`) to v10 (`fps_cap: FpsCap`).
/// If the file parses but lacks `fps_cap` and has the legacy `vsync` key,
/// infer the right cap from the bool.
fn migrate_v9_to_v10(text: &str) -> Option<GameConfig> {
    #[derive(serde::Deserialize)]
    struct V9Probe {
        version: u32,
        #[serde(default = "default_true")]
        vsync: bool,
    }
    fn default_true() -> bool {
        true
    }

    let probe: V9Probe = match ron::de::from_str(text) {
        Ok(p) => p,
        Err(_) => return None,
    };
    if probe.version >= 10 {
        return None;
    }
    // If the user already has `fps_cap` in their v9 config (manually added),
    // respect it. We detect by substring on the raw text since the
    // GameConfig struct always deserializes fps_cap (using the default).
    let has_fps_cap = text.contains("fps_cap");
    if has_fps_cap {
        return None;
    }
    let mut cfg: GameConfig = match ron::de::from_str(text) {
        Ok(c) => c,
        Err(_) => return None,
    };
    cfg.fps_cap = if probe.vsync {
        FpsCap::Vsync
    } else {
        FpsCap::Unlimited
    };
    cfg.version = 10;
    Some(cfg)
}

pub fn load_game_config() -> GameConfig {
    let path = game_config_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Some(migrated) = migrate_v9_to_v10(&text) {
            if let Err(err) = save_game_config(&migrated) {
                eprintln!("failed to save migrated config: {err}");
            }
            return migrated;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_v9_vsync_true_becomes_vsync() {
        let text = "(version: 9, vsync: true)\n";
        let cfg = migrate_v9_to_v10(text).expect("should migrate");
        assert_eq!(cfg.fps_cap, FpsCap::Vsync);
        assert_eq!(cfg.version, 10);
    }

    #[test]
    fn migrate_v9_vsync_false_becomes_unlimited() {
        let text = "(version: 9, vsync: false)\n";
        let cfg = migrate_v9_to_v10(text).expect("should migrate");
        assert_eq!(cfg.fps_cap, FpsCap::Unlimited);
        assert_eq!(cfg.version, 10);
    }

    #[test]
    fn migrate_skips_v10_configs() {
        let text = "(version: 10, fps_cap: Cap120, vsync: true)\n";
        assert!(migrate_v9_to_v10(text).is_none());
    }

    #[test]
    fn migrate_skips_when_fps_cap_already_present() {
        // Even with v9 + vsync: false, if fps_cap is in the text, the
        // user's own fps_cap takes precedence.
        let text = "(version: 9, vsync: false, fps_cap: Cap60)\n";
        assert!(migrate_v9_to_v10(text).is_none());
    }
}
