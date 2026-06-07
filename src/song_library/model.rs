use std::path::PathBuf;

use bevy::prelude::*;

use super::difficulty::pick_chart_index;

#[derive(Resource, Debug, Clone)]
pub struct SongLibrary {
    pub entries: Vec<SongEntry>,
    pub selected_entry: usize,
    pub selected_chart: usize,
    pub search: String,
}

#[derive(Debug, Clone)]
pub struct SongEntry {
    pub title: String,
    pub artist: Option<String>,
    pub folder: PathBuf,
    pub box_path: Vec<String>,
    pub preview_audio: Option<PathBuf>,
    pub preview_image: Option<PathBuf>,
    pub background_video: Option<PathBuf>,
    pub charts: Vec<ChartEntry>,
}

#[derive(Debug, Clone)]
pub struct ChartEntry {
    pub path: PathBuf,
    pub label: String,
    pub level: Option<f32>,
    pub bgm_path: Option<PathBuf>,
    pub bgm_volume: i32,
}

impl SongEntry {
    pub(crate) fn matches_search(&self, search: &str) -> bool {
        self.title.to_ascii_lowercase().contains(search)
            || self
                .artist
                .as_ref()
                .is_some_and(|artist| artist.to_ascii_lowercase().contains(search))
            || self
                .folder
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains(search)
            || self
                .box_path
                .iter()
                .any(|title| title.to_ascii_lowercase().contains(search))
            || self
                .charts
                .iter()
                .any(|chart| chart.label.to_ascii_lowercase().contains(search))
    }
}

impl SongLibrary {
    pub fn current_entry(&self) -> Option<&SongEntry> {
        self.entries.get(self.selected_entry)
    }

    pub fn current_chart(&self) -> Option<&ChartEntry> {
        self.current_entry().and_then(|entry| {
            entry.charts.get(
                self.selected_chart
                    .min(entry.charts.len().saturating_sub(1)),
            )
        })
    }

    pub fn select_chart_path(&mut self, path: &str) -> bool {
        let target = PathBuf::from(path);
        for (entry_idx, entry) in self.entries.iter().enumerate() {
            for (chart_idx, chart) in entry.charts.iter().enumerate() {
                if chart.path == target {
                    self.selected_entry = entry_idx;
                    self.selected_chart = chart_idx;
                    return true;
                }
            }
        }
        false
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        let search = self.search.trim().to_ascii_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if search.is_empty() || entry.matches_search(&search) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn selected_visible_index(&self) -> usize {
        self.visible_indices()
            .iter()
            .position(|index| *index == self.selected_entry)
            .unwrap_or(0)
    }

    pub fn apply_preferred_difficulty(&mut self, preferred: &str) {
        let Some(entry) = self.current_entry().cloned() else {
            return;
        };
        self.selected_chart = pick_chart_index(&entry.charts, preferred);
    }

    pub fn normalize_selection(&mut self, preferred: &str) {
        let visible = self.visible_indices();
        if let Some(first) = visible.first() {
            if !visible.contains(&self.selected_entry) {
                self.selected_entry = *first;
            }
            self.apply_preferred_difficulty(preferred);
        } else {
            self.selected_entry = 0;
            self.selected_chart = 0;
        }
    }

    pub fn select_next(&mut self, preferred: &str) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }
        let current = visible
            .iter()
            .position(|index| *index == self.selected_entry)
            .unwrap_or(0);
        self.selected_entry = visible[(current + 1).min(visible.len() - 1)];
        self.apply_preferred_difficulty(preferred);
    }

    pub fn select_previous(&mut self, preferred: &str) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }
        let current = visible
            .iter()
            .position(|index| *index == self.selected_entry)
            .unwrap_or(0);
        self.selected_entry = visible[current.saturating_sub(1)];
        self.apply_preferred_difficulty(preferred);
    }

    pub fn select_random(&mut self, index: usize, preferred: &str) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            return;
        }
        self.selected_entry = visible[index % visible.len()];
        self.apply_preferred_difficulty(preferred);
    }

    pub fn select_next_chart(&mut self) {
        self.selected_chart = (self.selected_chart + 1).min(self.max_chart_index());
    }

    pub fn select_previous_chart(&mut self) {
        self.selected_chart = self.selected_chart.saturating_sub(1);
    }

    fn max_chart_index(&self) -> usize {
        self.current_entry()
            .map(|entry| entry.charts.len().saturating_sub(1))
            .unwrap_or(0)
    }
}
