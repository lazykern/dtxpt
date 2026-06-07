use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::model::{ChartEntry, SongEntry, SongLibrary};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LibraryCache {
    version: u32,
    chart_root: String,
    root_mtime: u64,
    entries: Vec<CachedSongEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedSongEntry {
    title: String,
    artist: Option<String>,
    folder: String,
    box_path: Vec<String>,
    preview_audio: Option<String>,
    preview_image: Option<String>,
    background_video: Option<String>,
    charts: Vec<CachedChartEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedChartEntry {
    path: String,
    label: String,
    level: Option<f32>,
    #[serde(default)]
    bgm_path: Option<String>,
    #[serde(default = "default_bgm_volume")]
    bgm_volume: i32,
}

fn default_bgm_volume() -> i32 {
    100
}

const LIBRARY_CACHE_VERSION: u32 = 2;

pub fn load_cached_library(cache_path: &Path, chart_root: &str) -> Option<SongLibrary> {
    let text = std::fs::read_to_string(cache_path).ok()?;
    let cache = ron::from_str::<LibraryCache>(&text).ok()?;
    if cache.version != LIBRARY_CACHE_VERSION || cache.chart_root != chart_root {
        return None;
    }
    let current_mtime = chart_root_mtime(chart_root)?;
    if cache.root_mtime != current_mtime {
        return None;
    }
    Some(library_from_cache(cache))
}

pub fn save_library_cache(
    cache_path: &Path,
    chart_root: &str,
    library: &SongLibrary,
) -> Result<()> {
    let Some(root_mtime) = chart_root_mtime(chart_root) else {
        return Ok(());
    };
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cache = LibraryCache {
        version: LIBRARY_CACHE_VERSION,
        chart_root: chart_root.to_string(),
        root_mtime,
        entries: library.entries.iter().map(entry_to_cache).collect(),
    };
    let text = ron::ser::to_string_pretty(&cache, ron::ser::PrettyConfig::default())?;
    std::fs::write(cache_path, text)?;
    Ok(())
}

fn chart_root_mtime(chart_root: &str) -> Option<u64> {
    let metadata = std::fs::metadata(chart_root).ok()?;
    metadata
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn entry_to_cache(entry: &SongEntry) -> CachedSongEntry {
    CachedSongEntry {
        title: entry.title.clone(),
        artist: entry.artist.clone(),
        folder: entry.folder.to_string_lossy().into_owned(),
        box_path: entry.box_path.clone(),
        preview_audio: entry
            .preview_audio
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        preview_image: entry
            .preview_image
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        background_video: entry
            .background_video
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        charts: entry
            .charts
            .iter()
            .map(|chart| CachedChartEntry {
                path: chart.path.to_string_lossy().into_owned(),
                label: chart.label.clone(),
                level: chart.level,
                bgm_path: chart
                    .bgm_path
                    .as_ref()
                    .map(|path| path.to_string_lossy().into_owned()),
                bgm_volume: chart.bgm_volume,
            })
            .collect(),
    }
}

fn library_from_cache(cache: LibraryCache) -> SongLibrary {
    SongLibrary {
        entries: cache.entries.into_iter().map(entry_from_cache).collect(),
        selected_entry: 0,
        selected_chart: 0,
        search: String::new(),
    }
}

fn entry_from_cache(entry: CachedSongEntry) -> SongEntry {
    SongEntry {
        title: entry.title,
        artist: entry.artist,
        folder: PathBuf::from(entry.folder),
        box_path: entry.box_path,
        preview_audio: entry.preview_audio.map(PathBuf::from),
        preview_image: entry.preview_image.map(PathBuf::from),
        background_video: entry.background_video.map(PathBuf::from),
        charts: entry
            .charts
            .into_iter()
            .map(|chart| ChartEntry {
                path: PathBuf::from(chart.path),
                label: chart.label,
                level: chart.level,
                bgm_path: chart.bgm_path.map(PathBuf::from),
                bgm_volume: chart.bgm_volume,
            })
            .collect(),
    }
}
