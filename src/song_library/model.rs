use std::path::PathBuf;

use bevy::prelude::*;

use super::difficulty::pick_chart_index;

#[derive(Resource, Debug, Clone)]
pub struct SongLibrary {
    pub entries: Vec<SongEntry>,
    pub selected_entry: usize,
    pub selected_chart: usize,
    pub search: String,
    pub sort_mode: SongSortMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SongSortMode {
    #[default]
    Title,
    Artist,
    LevelDesc,
}

impl SongSortMode {
    pub fn next(self) -> Self {
        match self {
            Self::Title => Self::Artist,
            Self::Artist => Self::LevelDesc,
            Self::LevelDesc => Self::Title,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::LevelDesc => "Level ↓",
        }
    }
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
        let mut indices = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if search.is_empty() || entry.matches_search(&search) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.sort_indices(&mut indices);
        indices
    }

    fn sort_indices(&self, indices: &mut [usize]) {
        indices.sort_by(|a, b| {
            let left = &self.entries[*a];
            let right = &self.entries[*b];
            match self.sort_mode {
                SongSortMode::Title => left
                    .title
                    .to_ascii_lowercase()
                    .cmp(&right.title.to_ascii_lowercase()),
                SongSortMode::Artist => left
                    .artist
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .cmp(&right.artist.as_deref().unwrap_or("").to_ascii_lowercase())
                    .then_with(|| {
                        left.title
                            .to_ascii_lowercase()
                            .cmp(&right.title.to_ascii_lowercase())
                    }),
                SongSortMode::LevelDesc => best_level(right)
                    .total_cmp(&best_level(left))
                    .then_with(|| {
                        left.title
                            .to_ascii_lowercase()
                            .cmp(&right.title.to_ascii_lowercase())
                    }),
            }
        });
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
        self.select_random_with_sub_box(index, preferred, true);
    }

    pub fn select_random_with_sub_box(
        &mut self,
        index: usize,
        preferred: &str,
        descend_sub_boxes: bool,
    ) {
        let mut candidates = self.visible_indices();
        if !descend_sub_boxes
            && let Some(current_box) = self.current_entry().map(|entry| entry.box_path.clone())
        {
            candidates.retain(|candidate| {
                self.entries
                    .get(*candidate)
                    .is_some_and(|entry| entry.box_path == current_box)
            });
        }
        if candidates.is_empty() {
            return;
        }
        self.selected_entry = candidates[index % candidates.len()];
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

fn best_level(entry: &SongEntry) -> f32 {
    entry
        .charts
        .iter()
        .filter_map(|chart| chart.level)
        .max_by(f32::total_cmp)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart(label: &str, level: f32) -> ChartEntry {
        ChartEntry {
            path: PathBuf::from(format!("{label}.dtx")),
            label: label.into(),
            level: Some(level),
            bgm_path: None,
            bgm_volume: 100,
        }
    }

    fn entry(title: &str, artist: &str, box_path: &[&str], level: f32) -> SongEntry {
        SongEntry {
            title: title.into(),
            artist: Some(artist.into()),
            folder: PathBuf::from(title),
            box_path: box_path.iter().map(|item| item.to_string()).collect(),
            preview_audio: None,
            preview_image: None,
            background_video: None,
            charts: vec![chart("EXT", level)],
        }
    }

    fn library() -> SongLibrary {
        SongLibrary {
            entries: vec![
                entry("Beta", "Zed", &["Pack", "Sub"], 5.0),
                entry("Alpha", "Ann", &["Pack"], 9.0),
                entry("Gamma", "Moe", &["Pack", "Sub"], 7.0),
            ],
            selected_entry: 0,
            selected_chart: 0,
            search: String::new(),
            sort_mode: SongSortMode::Title,
        }
    }

    #[test]
    fn visible_indices_follow_sort_mode() {
        let mut library = library();
        assert_eq!(library.visible_indices(), vec![1, 0, 2]);

        library.sort_mode = SongSortMode::LevelDesc;
        assert_eq!(library.visible_indices(), vec![1, 2, 0]);
    }

    #[test]
    fn random_select_can_stay_within_current_sub_box() {
        let mut library = library();
        library.selected_entry = 0;
        library.select_random_with_sub_box(1, "", false);
        assert_eq!(library.selected_entry, 2);

        library.select_random_with_sub_box(0, "", true);
        assert_eq!(library.selected_entry, 1);
    }
}
