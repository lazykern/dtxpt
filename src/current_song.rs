use std::path::{Path, PathBuf};

use bevy::prelude::*;

use crate::config::{GameConfig, save_game_config};
use crate::gameplay::SelectedChartPath;
use dtxpt::song_library::SongLibrary;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct CurrentSong {
    pub title: String,
    pub artist: Option<String>,
    pub chart_path: String,
    pub chart_label: String,
    pub bgm_path: Option<PathBuf>,
    pub bgm_volume: i32,
    pub preview_image: Option<PathBuf>,
}

impl Default for CurrentSong {
    fn default() -> Self {
        Self::from_path_stub("")
    }
}

impl CurrentSong {
    pub fn from_path_stub(path: &str) -> Self {
        let title = Path::new(path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .filter(|stem| !stem.is_empty())
            .unwrap_or_else(|| "No chart selected".into());
        Self {
            title,
            artist: None,
            chart_path: path.to_string(),
            chart_label: String::new(),
            bgm_path: None,
            bgm_volume: 100,
            preview_image: None,
        }
    }

    pub fn from_library(library: &SongLibrary) -> Option<Self> {
        let entry = library.current_entry()?;
        let chart = library.current_chart()?;
        Some(Self {
            title: entry.title.clone(),
            artist: entry.artist.clone(),
            chart_path: chart.path.to_string_lossy().to_string(),
            chart_label: chart.label.clone(),
            bgm_path: chart.bgm_path.clone(),
            bgm_volume: chart.bgm_volume,
            preview_image: entry.preview_image.clone(),
        })
    }

    pub fn display_line(&self) -> Option<String> {
        if self.chart_path.is_empty() {
            return None;
        }
        let artist = self.artist.as_deref().unwrap_or("").trim();
        if artist.is_empty() {
            Some(self.title.clone())
        } else {
            Some(format!("{} — {}", self.title, artist))
        }
    }

    pub fn sync_selected_chart_path(&self, selected: &mut SelectedChartPath) {
        selected.0 = self.chart_path.clone();
    }

    pub fn persist_last_chart(&self, config: &mut GameConfig) {
        if self.chart_path.is_empty() {
            return;
        }
        let mut changed = false;
        if config.last_chart_path != self.chart_path {
            config.last_chart_path = self.chart_path.clone();
            changed = true;
        }
        if !self.chart_label.is_empty() && config.preferred_difficulty != self.chart_label {
            config.preferred_difficulty = self.chart_label.clone();
            changed = true;
        }
        if !changed {
            return;
        }
        if let Err(err) = save_game_config(config) {
            warn!("failed to persist last chart: {err}");
        }
    }
}

pub fn apply_library_selection(current: &mut CurrentSong, library: &SongLibrary) {
    if let Some(next) = CurrentSong::from_library(library) {
        *current = next;
    }
}

pub fn align_library_to_current_song(
    library: &mut SongLibrary,
    current: &CurrentSong,
    preferred: &str,
) {
    if current.chart_path.is_empty() {
        if !library.entries.is_empty() {
            library.normalize_selection(preferred);
        }
        return;
    }
    if !library.select_chart_path(&current.chart_path) {
        library.normalize_selection(preferred);
    }
}

pub fn enrich_current_song_from_library(current: &mut CurrentSong, library: &mut SongLibrary) {
    if current.chart_path.is_empty() {
        return;
    }
    if !library.select_chart_path(&current.chart_path) {
        return;
    }
    if let Some(next) = CurrentSong::from_library(library) {
        *current = next;
    }
}
