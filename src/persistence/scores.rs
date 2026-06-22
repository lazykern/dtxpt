use std::collections::HashMap;

use anyhow::Result;
use bevy::prelude::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::config::store::project_dirs;
use crate::gameplay::RunResult;

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ScoreStore {
    pub scores: HashMap<String, BestScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestScore {
    pub score: u32,
    pub accuracy: f32,
    pub max_combo: u32,
    pub perfect: u32,
    pub great: u32,
    pub good: u32,
    pub poor: u32,
    pub miss: u32,
    #[serde(default)]
    pub history: [String; 5],
    /// Single-instrument best rank (typically the drums rank, since
    /// dtxpt's first-pass runs defaulted to drum score persistence).
    /// Kept for backward-compat with v1 `scores.ron` files.
    #[serde(default)]
    pub rank: String,
    /// Per-instrument best ranks from BocuD `[File]` section. Newer
    /// `score.ini` codec writes these alongside the legacy `rank`.
    #[serde(default)]
    pub rank_drums: String,
    #[serde(default)]
    pub rank_guitar: String,
    #[serde(default)]
    pub rank_bass: String,
}

impl ScoreStore {
    pub fn best_for_path(&self, path: impl AsRef<std::path::Path>) -> Option<&BestScore> {
        self.scores
            .get(&path.as_ref().to_string_lossy().to_string())
    }
}

impl BestScore {
    pub fn instrument_ranks(&self) -> [(&'static str, &str); 3] {
        [
            ("Drums", self.rank_drums.as_str()),
            ("Guitar", self.rank_guitar.as_str()),
            ("Bass", self.rank_bass.as_str()),
        ]
    }
    pub fn from_result(result: &RunResult) -> Self {
        Self {
            score: result.score,
            accuracy: result.accuracy,
            max_combo: result.max_combo,
            perfect: result.perfect,
            great: result.great,
            good: result.good,
            poor: result.poor,
            miss: result.miss,
            history: Default::default(),
            rank: result.rank.clone(),
            rank_drums: result.rank.clone(),
            rank_guitar: String::new(),
            rank_bass: String::new(),
        }
    }

    pub fn beats(&self, result: &RunResult) -> bool {
        result.score > self.score
            || (result.score == self.score && result.accuracy > self.accuracy)
            || (result.score == self.score
                && (result.accuracy - self.accuracy).abs() < f32::EPSILON
                && result.max_combo > self.max_combo)
    }
}

pub fn score_store_path() -> std::path::PathBuf {
    project_dirs()
        .map(|dirs: ProjectDirs| dirs.data_local_dir().join("scores.ron"))
        .unwrap_or_else(|| std::path::PathBuf::from("scores.ron"))
}

pub fn load_score_store() -> ScoreStore {
    let path = score_store_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        match ron::de::from_str::<ScoreStore>(&text) {
            Ok(store) => return store,
            Err(err) => eprintln!("failed to parse {}: {err}", path.display()),
        }
    }
    ScoreStore::default()
}

pub fn save_score_store(store: &ScoreStore) -> Result<()> {
    let path = score_store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = ron::ser::to_string_pretty(store, ron::ser::PrettyConfig::default())?;
    std::fs::write(path, text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instrument_ranks_returns_per_instrument_labels() {
        let score = BestScore {
            score: 0,
            accuracy: 0.0,
            max_combo: 0,
            perfect: 0,
            great: 0,
            good: 0,
            poor: 0,
            miss: 0,
            history: Default::default(),
            rank: "S".into(),
            rank_drums: "SS".into(),
            rank_guitar: "A".into(),
            rank_bass: String::new(),
        };
        let ranks = score.instrument_ranks();
        assert_eq!(ranks[0], ("Drums", "SS"));
        assert_eq!(ranks[1], ("Guitar", "A"));
        assert_eq!(ranks[2], ("Bass", ""));
    }
}
