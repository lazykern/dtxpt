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
}

impl ScoreStore {
    pub fn best_for_path(&self, path: impl AsRef<std::path::Path>) -> Option<&BestScore> {
        self.scores
            .get(&path.as_ref().to_string_lossy().to_string())
    }
}

impl BestScore {
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
