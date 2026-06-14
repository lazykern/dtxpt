use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};

use crate::chart::dtx::{command_index, parse_directive, read_text, resolve_chart_bgm};

use super::difficulty::compare_difficulty_labels;
use super::model::{ChartEntry, SongEntry, SongLibrary};

#[derive(Debug, Clone, Default)]
struct DtxMetadata {
    title: Option<String>,
    artist: Option<String>,
    level: Option<f32>,
    preview_audio: Option<PathBuf>,
    preview_image: Option<PathBuf>,
    background_video: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
struct BoxMetadata {
    title: Option<String>,
}

pub fn scan_song_library(chart_root: &str) -> SongLibrary {
    let mut roots = Vec::new();
    let chart_root = PathBuf::from(chart_root);
    if chart_root.exists() {
        roots.push(chart_root);
    }

    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    for root in roots {
        scan_root(&root, &mut seen, &mut entries);
    }

    entries.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    SongLibrary {
        entries,
        selected_entry: 0,
        selected_chart: 0,
        search: String::new(),
    }
}

fn scan_root(root: &Path, seen: &mut HashSet<PathBuf>, entries: &mut Vec<SongEntry>) {
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let Ok(read_dir) = std::fs::read_dir(&dir) else {
            continue;
        };

        let mut loose_dtx = Vec::new();
        let mut set_def = None;
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if should_descend(&path) {
                    dirs.push(path);
                }
                continue;
            }

            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if name == "set.def" {
                set_def = Some(path);
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("dtx"))
            {
                loose_dtx.push(path);
            }
        }

        if let Some(set_def) = set_def
            && let Ok(entry) = set_def_entry(&set_def, root, seen)
        {
            entries.push(entry);
            continue;
        }

        for path in loose_dtx {
            if seen.insert(normalize_path(&path))
                && let Ok(entry) = loose_dtx_entry(&path, root)
            {
                entries.push(entry);
            }
        }
    }
}

fn should_descend(path: &Path) -> bool {
    let Some(name) = path.file_name().map(|name| name.to_string_lossy()) else {
        return true;
    };
    !matches!(name.as_ref(), ".git" | "target" | "references")
}

fn set_def_entry(path: &Path, root: &Path, seen: &mut HashSet<PathBuf>) -> Result<SongEntry> {
    let text = read_text(path)?;
    let folder = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut title = folder
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string());
    let mut labels = std::collections::HashMap::<usize, String>::new();
    let mut files = std::collections::HashMap::<usize, String>::new();

    for raw in text.lines() {
        let Some((command, value)) = parse_directive(raw) else {
            continue;
        };
        if command.eq_ignore_ascii_case("TITLE") {
            title = value.to_string();
        } else if let Some(index) = command_index(&command, "L", "LABEL") {
            labels.insert(index, value.to_string());
        } else if let Some(index) = command_index(&command, "L", "FILE") {
            files.insert(index, value.to_string());
        }
    }

    let mut charts = files
        .into_iter()
        .filter_map(|(index, file)| {
            let path = folder.join(file);
            if !path.exists() || !seen.insert(normalize_path(&path)) {
                return None;
            }
            let metadata = read_dtx_metadata(&path).unwrap_or_default();
            Some(chart_entry_from_path(
                path,
                labels
                    .get(&index)
                    .cloned()
                    .unwrap_or_else(|| default_difficulty_label(index).to_string()),
                metadata.level,
            ))
        })
        .collect::<Vec<_>>();

    charts.sort_by(|a, b| compare_difficulty_labels(&a.label, &b.label));
    if charts.is_empty() {
        return Err(anyhow!(
            "set.def has no existing DTX files: {}",
            path.display()
        ));
    }

    let first_meta = charts
        .first()
        .and_then(|chart| read_dtx_metadata(&chart.path).ok())
        .unwrap_or_default();

    Ok(SongEntry {
        title: first_meta.title.unwrap_or(title),
        artist: first_meta.artist,
        box_path: box_path_for(&folder, root),
        preview_audio: first_meta
            .preview_audio
            .or_else(|| find_media_file(&folder, &["pre", "preview"], &["ogg", "wav", "mp3"])),
        preview_image: first_meta.preview_image.or_else(|| {
            find_media_file(
                &folder,
                &["pre", "preview", "img", "jacket"],
                &["png", "jpg", "jpeg"],
            )
        }),
        background_video: first_meta
            .background_video
            .or_else(|| find_media_file(&folder, &["bg", "movie"], &["mp4", "avi", "mpg"])),
        folder,
        charts,
    })
}

fn loose_dtx_entry(path: &Path, root: &Path) -> Result<SongEntry> {
    let metadata = read_dtx_metadata(path)?;
    let folder = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let title = metadata.title.unwrap_or_else(|| {
        path.file_stem()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    });
    let label = path
        .file_stem()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "DTX".to_string());

    Ok(SongEntry {
        title,
        artist: metadata.artist,
        box_path: box_path_for(&folder, root),
        preview_audio: metadata
            .preview_audio
            .or_else(|| find_media_file(&folder, &["pre", "preview"], &["ogg", "wav", "mp3"])),
        preview_image: metadata.preview_image.or_else(|| {
            find_media_file(
                &folder,
                &["pre", "preview", "img", "jacket"],
                &["png", "jpg", "jpeg"],
            )
        }),
        background_video: metadata
            .background_video
            .or_else(|| find_media_file(&folder, &["bg", "movie"], &["mp4", "avi", "mpg"])),
        folder,
        charts: vec![chart_entry_from_path(
            path.to_path_buf(),
            label,
            metadata.level,
        )],
    })
}

fn read_dtx_metadata(path: &Path) -> Result<DtxMetadata> {
    let text = read_text(path)?;
    let mut metadata = DtxMetadata::default();
    for raw in text.lines() {
        let Some((command, value)) = parse_directive(raw) else {
            continue;
        };
        if command.eq_ignore_ascii_case("TITLE") {
            metadata.title = Some(value.to_string());
        } else if command.eq_ignore_ascii_case("ARTIST") {
            metadata.artist = Some(value.to_string());
        } else if command.eq_ignore_ascii_case("DLEVEL") {
            // DTXMania: integer DLEVEL = level * 10 (e.g., 85 = 8.5)
            // Decimal notation (contains '.') used directly
            metadata.level = if value.contains('.') {
                value.parse::<f32>().ok()
            } else {
                value.parse::<f32>().ok().map(|v| v / 10.0)
            };
        } else if command.eq_ignore_ascii_case("PREVIEW") {
            metadata.preview_audio = resolve_media_path(path, value);
        } else if command.eq_ignore_ascii_case("PREIMAGE") {
            metadata.preview_image = resolve_media_path(path, value);
        } else if command.eq_ignore_ascii_case("PREMOVIE") || command.eq_ignore_ascii_case("AVIZZ")
        {
            metadata.background_video = resolve_media_path(path, value);
        }
    }
    Ok(metadata)
}

fn read_box_metadata(path: &Path) -> Result<BoxMetadata> {
    let text = read_text(path)?;
    let mut metadata = BoxMetadata::default();
    for raw in text.lines() {
        let Some((command, value)) = parse_directive(raw) else {
            continue;
        };
        if command.eq_ignore_ascii_case("TITLE") {
            metadata.title = Some(value.to_string());
        }
    }
    Ok(metadata)
}

fn box_path_for(folder: &Path, root: &Path) -> Vec<String> {
    let Ok(relative) = folder.strip_prefix(root) else {
        return Vec::new();
    };

    let mut path = root.to_path_buf();
    let mut boxes = Vec::new();
    for component in relative.components() {
        path.push(component.as_os_str());
        let Some(box_def) = find_named_file(&path, "box.def") else {
            continue;
        };
        if let Ok(metadata) = read_box_metadata(&box_def)
            && let Some(title) = metadata.title.filter(|title| !title.is_empty())
        {
            boxes.push(title);
        }
    }
    boxes
}

fn find_named_file(dir: &Path, target: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        let name = path.file_name()?.to_string_lossy();
        (path.is_file() && name.eq_ignore_ascii_case(target)).then_some(path)
    })
}

fn resolve_media_path(chart_path: &Path, value: &str) -> Option<PathBuf> {
    let value = value.trim().trim_matches('"');
    if value.is_empty() {
        return None;
    }
    let folder = chart_path.parent().unwrap_or_else(|| Path::new("."));
    let direct = folder.join(value);
    if direct.exists() {
        return Some(direct);
    }
    find_named_file(folder, value)
}

fn find_media_file(dir: &Path, stems: &[&str], extensions: &[&str]) -> Option<PathBuf> {
    std::fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        if !path.is_file() {
            return None;
        }
        let stem = path.file_stem()?.to_string_lossy();
        let ext = path.extension()?.to_string_lossy();
        let stem_matches = stems.iter().any(|target| stem.eq_ignore_ascii_case(target));
        let ext_matches = extensions
            .iter()
            .any(|target| ext.eq_ignore_ascii_case(target));
        (stem_matches && ext_matches).then_some(path)
    })
}

fn default_difficulty_label(index: usize) -> &'static str {
    match index {
        1 => "BASIC",
        2 => "ADVANCED",
        3 => "EXTREME",
        4 => "MASTER",
        5 => "DTXMANIA",
        _ => "DTX",
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn chart_entry_from_path(path: PathBuf, label: String, level: Option<f32>) -> ChartEntry {
    let bgm = resolve_chart_bgm(&path);
    ChartEntry {
        path,
        label,
        level,
        bgm_path: bgm.as_ref().map(|bgm| bgm.path.clone()),
        bgm_volume: bgm.map(|bgm| bgm.volume).unwrap_or(100),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_set_def_indices() {
        assert_eq!(command_index("L1LABEL", "L", "LABEL"), Some(1));
        assert_eq!(command_index("L4FILE", "L", "FILE"), Some(4));
        assert_eq!(command_index("TITLE", "L", "FILE"), None);
    }

    #[test]
    fn parses_directive_with_colon_or_space() {
        assert_eq!(parse_directive("#TITLE: Song").unwrap().1, "Song");
        assert_eq!(parse_directive("#ARTIST Band").unwrap().1, "Band");
        assert!(parse_directive("; nope").is_none());
    }

    #[test]
    fn collects_box_path_from_ancestor_box_defs() {
        let root =
            std::env::temp_dir().join(format!("dtxpt-song-library-test-{}", std::process::id()));
        let box_dir = root.join("Pack");
        let song_dir = box_dir.join("Song");
        std::fs::create_dir_all(&song_dir).unwrap();
        std::fs::write(box_dir.join("BOX.def"), "#TITLE: Test Pack\n").unwrap();
        std::fs::write(
            song_dir.join("song.dtx"),
            "#TITLE: Test Song\n#DLEVEL: 4.2\n#PREVIEW: pre.ogg\n#PREIMAGE: img.png\n",
        )
        .unwrap();
        std::fs::write(song_dir.join("pre.ogg"), b"placeholder").unwrap();
        std::fs::write(song_dir.join("img.png"), b"placeholder").unwrap();

        let entry = loose_dtx_entry(&song_dir.join("song.dtx"), &root).unwrap();
        assert_eq!(entry.box_path, vec!["Test Pack"]);
        assert_eq!(entry.preview_audio, Some(song_dir.join("pre.ogg")));
        assert_eq!(entry.preview_image, Some(song_dir.join("img.png")));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dlevel_integer_gets_divided_by_10() {
        let dir = std::env::temp_dir().join(format!("dtxpt-dlevel-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.dtx");
        std::fs::write(&path, "#TITLE: Test\n#DLEVEL: 85\n").unwrap();
        let meta = read_dtx_metadata(&path).unwrap();
        assert_eq!(meta.level, Some(8.5));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn dlevel_decimal_used_directly() {
        let dir =
            std::env::temp_dir().join(format!("dtxpt-dlevel-dec-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.dtx");
        std::fs::write(&path, "#TITLE: Test\n#DLEVEL: 4.2\n").unwrap();
        let meta = read_dtx_metadata(&path).unwrap();
        assert_eq!(meta.level, Some(4.2));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
