use std::path::Path;

use anyhow::{Result, anyhow};

use crate::chart::dtx::parse_dtx_chart_with_compute_mode;
use crate::chart::dtx::text::decode_bytes;
use crate::chart::model::Chart;
use crate::chart::timing::{ChartTiming, ChipPlayTimeComputeMode};

pub fn load_chart_from_path(path: &str) -> Result<(Chart, ChartTiming)> {
    load_chart_from_path_with_compute_mode(path, ChipPlayTimeComputeMode::default())
}

pub fn load_chart_from_path_with_compute_mode(
    path: &str,
    chip_play_time_compute_mode: ChipPlayTimeComputeMode,
) -> Result<(Chart, ChartTiming)> {
    let bytes = std::fs::read(path).map_err(|err| {
        let hint = suggest_dtx_files(path);
        anyhow!("chart file not found: {path} ({err}){hint}")
    })?;

    let text = decode_bytes(&bytes);
    let chart_dir = Path::new(path)
        .parent()
        .and_then(|p| std::fs::canonicalize(p).ok())
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    parse_dtx_chart_with_compute_mode(&text, path, &chart_dir, chip_play_time_compute_mode)
        .map_err(|err| anyhow!("failed to parse DTX '{path}': {err}"))
}

fn suggest_dtx_files(path: &str) -> String {
    let path = Path::new(path);
    let Some(parent) = path.parent() else {
        return String::new();
    };
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut matches: Vec<String> = std::fs::read_dir(parent)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".dtx"))
        .filter(|name| stem.is_empty() || name.contains(&stem))
        .collect();
    matches.sort();

    if matches.is_empty() {
        return String::new();
    }

    format!("\n\ndid you mean one of these?\n  {}", matches.join("\n  "))
}
