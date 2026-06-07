use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::channels::dtx_wav_volume_command_id;
use super::text::{parse_directive, read_text};
use super::util::{base36_pair, base36_str, normalized_pairs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartBgm {
    pub path: PathBuf,
    pub volume: i32,
}

pub fn resolve_chart_bgm(chart_path: &Path) -> Option<ChartBgm> {
    let text = read_text(chart_path).ok()?;
    let chart_dir = chart_path.parent()?;
    let mut wav_files: HashMap<u32, String> = HashMap::new();
    let mut wav_volumes: HashMap<u32, i32> = HashMap::new();
    let mut bgm_wav = None;
    let mut first_bgm_event = None;

    for raw in text.lines() {
        let Some((command, value)) = parse_directive(raw) else {
            continue;
        };

        if command.len() == 5 && command[..3].eq_ignore_ascii_case("WAV") {
            if let Ok(id) = base36_str(&command[3..5]) {
                wav_files.insert(id, value.to_string());
            }
            continue;
        }
        if let Some(id) = dtx_wav_volume_command_id(&command) {
            if let Ok(volume) = value.parse::<i32>() {
                wav_volumes.insert(id, volume.clamp(0, 100));
            }
            continue;
        }
        if command.eq_ignore_ascii_case("BGMWAV") {
            if let Ok(id) = base36_str(value) {
                bgm_wav = Some(id);
            }
            continue;
        }
        if command.len() == 5 && command[..3].chars().all(|c| c.is_ascii_digit()) {
            let channel = u32::from_str_radix(&command[3..5], 16).ok()?;
            if channel != 0x01 {
                continue;
            }
            let pairs = normalized_pairs(value);
            for pair in pairs.chunks(2) {
                if pair.len() != 2 || pair == b"00" {
                    continue;
                }
                if let Ok(wav) = base36_pair(pair) {
                    first_bgm_event.get_or_insert(wav);
                }
            }
        }
    }

    let wav_id = bgm_wav.or(first_bgm_event)?;
    let filename = wav_files.get(&wav_id)?;
    let path = resolve_chart_asset_path(chart_dir, filename);
    if !path.exists() {
        return None;
    }
    Some(ChartBgm {
        path,
        volume: *wav_volumes.get(&wav_id).unwrap_or(&100),
    })
}

fn resolve_chart_asset_path(chart_dir: &Path, fname: &str) -> PathBuf {
    let direct = chart_dir.join(fname);
    if direct.exists() {
        return direct;
    }
    if let Ok(entries) = std::fs::read_dir(chart_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case(fname))
            {
                return path;
            }
        }
    }
    direct
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_bgm_from_bgmwav_and_channel() {
        let dir = std::env::temp_dir().join(format!("dtxpt-bgm-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("stage.ogg"), b"bgm").unwrap();
        std::fs::write(
            dir.join("chart.dtx"),
            "#TITLE: Test\n#WAV01: stage.ogg\n#BGMWAV: 01\n#00101:0100000001000000\n",
        )
        .unwrap();

        let bgm = resolve_chart_bgm(&dir.join("chart.dtx")).unwrap();
        assert_eq!(bgm.path, dir.join("stage.ogg"));
        assert_eq!(bgm.volume, 100);

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn strips_inline_comments_from_wav_filename() {
        let dir = std::env::temp_dir().join(format!("dtxpt-bgm-comment-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("bgm_d.ogg"), b"bgm").unwrap();
        std::fs::write(
            dir.join("chart.dtx"),
            "#WAV0X: bgm_d.ogg\t;BGM\n#BGMWAV: 0X\n",
        )
        .unwrap();

        let bgm = resolve_chart_bgm(&dir.join("chart.dtx")).unwrap();
        assert_eq!(bgm.path, dir.join("bgm_d.ogg"));

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn returns_none_when_no_bgm() {
        let dir = std::env::temp_dir().join(format!("dtxpt-bgm-none-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("chart.dtx"), "#TITLE: Test\n#00011:01010101\n").unwrap();
        assert!(resolve_chart_bgm(&dir.join("chart.dtx")).is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
