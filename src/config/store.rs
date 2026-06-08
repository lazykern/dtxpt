use std::path::Path;

use anyhow::Result;
use directories::ProjectDirs;

use crate::gameplay::mods::AutoMode;
use dtxpt::input::bindings::DrumLane;

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

/// Migrate a v10 config (had `play_mode: PlayMode`) to v11
/// (`per_lane_auto: BTreeSet<DrumLane>` + `auto_mode: AutoMode`).
/// Legacy `play_mode = Auto` becomes `per_lane_auto = ALL_LANES` so the
/// user's effective "all auto" intent is preserved as their saved
/// per-lane config. Legacy `play_mode = Practice` is dropped (Practice
/// becomes a top-level mode, picked at song start).
fn migrate_v10_to_v11(text: &str) -> Option<GameConfig> {
    // v10 has a `play_mode: PlayMode` field that v11 removed. RON fails
    // to parse the v10 text into a v11 `GameConfig` because `play_mode`
    // is an unknown field. Probe the text with a v10-specific struct
    // that only knows the fields we care about for migration, then
    // construct a fresh v11 config from the probe data + v11 defaults.
    //
    // `play_mode` is deserialized as a local enum with the same variant
    // names as the v10 `PlayMode`. This is needed because RON serializes
    // enum variants as bare identifiers (e.g. `Auto`), not as strings.
    #[derive(serde::Deserialize, Default)]
    #[serde(default)]
    struct V10Probe {
        version: u32,
        play_mode: V10PlayMode,
    }

    #[derive(serde::Deserialize, Default)]
    enum V10PlayMode {
        #[default]
        Normal,
        Practice,
        Auto,
    }

    let probe: V10Probe = match ron::de::from_str(text) {
        Ok(p) => p,
        Err(_) => return None,
    };
    if probe.version >= 11 {
        return None;
    }
    if !text.contains("play_mode") {
        return None;
    }
    let was_auto = matches!(probe.play_mode, V10PlayMode::Auto);
    let mut cfg = GameConfig::default();
    if was_auto {
        cfg.per_lane_auto = DrumLane::ALL.iter().copied().collect();
        cfg.auto_mode = AutoMode::PerLane;
    }
    cfg.version = 11;
    Some(cfg)
}

pub fn load_game_config() -> GameConfig {
    let path = game_config_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Some(migrated) = migrate_v10_to_v11(&text) {
            if let Err(err) = save_game_config(&migrated) {
                eprintln!("failed to save migrated v10->v11 config: {err}");
            }
            return migrated;
        }
        if let Some(migrated) = migrate_v9_to_v10(&text) {
            if let Err(err) = save_game_config(&migrated) {
                eprintln!("failed to save migrated v9->v10 config: {err}");
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

    #[test]
    fn migrate_v10_auto_becomes_per_lane_all_ten() {
        let text = "(version: 10, play_mode: Auto, fps_cap: Vsync)\n";
        let cfg = migrate_v10_to_v11(text).expect("should migrate");
        assert_eq!(cfg.version, 11);
        // Legacy Auto is preserved as the user's per-lane config so their
        // effective "all 10 auto" intent is kept.
        assert_eq!(cfg.per_lane_auto.len(), DrumLane::ALL.len());
        assert_eq!(cfg.auto_mode, AutoMode::PerLane);
    }

    #[test]
    fn migrate_v10_normal_keeps_empty_per_lane() {
        let text = "(version: 10, play_mode: Normal, fps_cap: Vsync)\n";
        let cfg = migrate_v10_to_v11(text).expect("should migrate");
        assert_eq!(cfg.version, 11);
        assert!(cfg.per_lane_auto.is_empty());
        assert_eq!(cfg.auto_mode, AutoMode::PerLane);
    }

    #[test]
    fn migrate_v10_practice_does_not_touch_per_lane() {
        let text = "(version: 10, play_mode: Practice, fps_cap: Vsync)\n";
        let cfg = migrate_v10_to_v11(text).expect("should migrate");
        assert_eq!(cfg.version, 11);
        assert!(cfg.per_lane_auto.is_empty());
    }

    #[test]
    fn migrate_skips_v11_configs() {
        let text = "(version: 11, per_lane_auto: [], auto_mode: PerLane)\n";
        assert!(migrate_v10_to_v11(text).is_none());
    }
}
