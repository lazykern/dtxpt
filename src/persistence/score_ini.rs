use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{Datelike, Local};
use encoding_rs::SHIFT_JIS;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use dtxpt::input::bindings::DrumLane;

use crate::config::HitSoundPriority;
use crate::gameplay::RunResult;

use super::BestScore;

impl PerChartScore {
    /// Returns the play speed as a "N/D" string for the score.ini PlaySpeed field.
    pub fn play_speed_formatted(&self) -> String {
        format!("{}/{}", self.play_speed_num, self.play_speed_den)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PerChartScore {
    pub chart_hash: String,
    pub section_hash: String,
    pub play_count_drums: u32,
    pub clear_count_drums: u32,
    pub history_count: u32,
    pub history: [String; 5],
    pub date_time: String,
    pub progress: String,
    pub score: u32,
    pub play_skill: f32,
    pub skill: f32,
    pub perfect: u32,
    pub great: u32,
    pub good: u32,
    pub poor: u32,
    pub miss: u32,
    pub max_combo: u32,
    pub total_chips: u32,
    pub auto_play: String,
    pub play_speed_num: u32,
    pub play_speed_den: u32,
    pub use_keyboard: bool,
    pub use_midi_in: bool,
    pub use_joypad: bool,
    pub use_mouse: bool,
    pub hit_sound_priority_hh: HitSoundPriority,
    pub hit_sound_priority_ft: HitSoundPriority,
    pub hit_sound_priority_cy: HitSoundPriority,
    pub cleared: bool,
    pub rank: String,
    /// Per-instrument best ranks from BocuD `[File]` section. BocuD
    /// writes three fields — `BestRankDrums` (also surfaced as `rank`
    /// above), `BestRankGuitar`, `BestRankBass`. We carry the other two
    /// here so the result screen can show all three panels.
    /// (`references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CDTX.cs` `[File]`
    /// section.)
    #[serde(default)]
    pub rank_guitar: String,
    #[serde(default)]
    pub rank_bass: String,
}

impl PerChartScore {
    pub fn from_result(result: &RunResult) -> Self {
        Self {
            chart_hash: compute_file_md5(&result.chart_path).unwrap_or_default(),
            section_hash: String::new(),
            play_count_drums: 0,
            clear_count_drums: 0,
            history_count: 0,
            history: Default::default(),
            date_time: bocud_datetime_now(),
            progress: result.progress.clone(),
            score: result.score,
            play_skill: result.play_skill,
            skill: result.game_skill,
            perfect: result.perfect,
            great: result.great,
            good: result.good,
            poor: result.poor,
            miss: result.miss,
            max_combo: result.max_combo,
            total_chips: result.perfect + result.great + result.good + result.poor + result.miss,
            auto_play: autoplay_string(&result.auto_lanes),
            play_speed_num: result.play_speed_num,
            play_speed_den: result.play_speed_den,
            use_keyboard: result.used_keyboard,
            use_midi_in: result.used_midi_in,
            use_joypad: result.used_joypad,
            use_mouse: result.used_mouse,
            hit_sound_priority_hh: result.hit_sound_priority_hh,
            hit_sound_priority_ft: result.hit_sound_priority_ft,
            hit_sound_priority_cy: result.hit_sound_priority_cy,
            cleared: result.cleared,
            rank: result.rank.clone(),
            rank_guitar: String::new(),
            rank_bass: String::new(),
        }
    }

    pub fn to_best_score(&self) -> BestScore {
        let total = self.perfect + self.great + self.good + self.poor + self.miss;
        let accuracy = if total == 0 {
            0.0
        } else {
            let units = self.perfect as f32 + self.great as f32;
            100.0 * units / total as f32
        };
        BestScore {
            score: self.score,
            accuracy,
            max_combo: self.max_combo,
            perfect: self.perfect,
            great: self.great,
            good: self.good,
            poor: self.poor,
            miss: self.miss,
            history: self.history.clone(),
            rank: self.rank.clone(),
            rank_drums: self.rank.clone(),
            rank_guitar: self.rank_guitar.clone(),
            rank_bass: self.rank_bass.clone(),
        }
    }
}

pub fn score_ini_path(chart_path: impl AsRef<Path>) -> PathBuf {
    let mut path = chart_path.as_ref().as_os_str().to_os_string();
    path.push(".score.ini");
    PathBuf::from(path)
}

pub fn read_score_ini(path: impl AsRef<Path>) -> Result<Option<PerChartScore>> {
    let bytes = match std::fs::read(path.as_ref()) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    let (text, _, _) = SHIFT_JIS.decode(&bytes);
    Ok(parse_score_ini(&text))
}

pub fn write_score_ini(path: impl AsRef<Path>, score: &PerChartScore) -> Result<()> {
    write_score_ini_text(path, &render_score_ini(score))
}

pub fn write_score_ini_result(path: impl AsRef<Path>, result: &RunResult) -> Result<()> {
    let path = path.as_ref();
    let mut current = PerChartScore::from_result(result);
    let existing = read_score_ini_records(path)?.unwrap_or_else(|| ScoreIniRecords {
        hi_score: current.clone(),
        hi_skill: current.clone(),
    });
    copy_history_with_new_line(&existing.hi_score, &mut current, result);
    let hi_score = if score_beats(&current, &existing.hi_score) {
        current.clone()
    } else {
        existing.hi_score
    };
    let hi_skill = if current.skill > existing.hi_skill.skill {
        current.clone()
    } else {
        existing.hi_skill
    };
    let text = render_score_ini_parts(&hi_score, &hi_score, &hi_skill, &current);
    write_score_ini_text(path, &text)
}

fn write_score_ini_text(path: impl AsRef<Path>, text: &str) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    let (bytes, _, _) = SHIFT_JIS.encode(text);
    std::fs::write(path, bytes.as_ref())?;
    Ok(())
}

struct ScoreIniRecords {
    hi_score: PerChartScore,
    hi_skill: PerChartScore,
}

fn read_score_ini_records(path: impl AsRef<Path>) -> Result<Option<ScoreIniRecords>> {
    let bytes = match std::fs::read(path.as_ref()) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    let (text, _, _) = SHIFT_JIS.decode(&bytes);
    Ok(parse_score_ini_records(&text))
}

fn parse_score_ini(text: &str) -> Option<PerChartScore> {
    parse_score_ini_records(text).map(|records| records.hi_score)
}

fn parse_score_ini_records(text: &str) -> Option<ScoreIniRecords> {
    let sections = parse_sections(text);
    let file = sections.get("File");
    let hi_score = parse_score_ini_section(file, sections.get("HiScore.Drums")?)?;
    let hi_skill = sections
        .get("HiSkill.Drums")
        .and_then(|drums| parse_score_ini_section(file, drums))
        .unwrap_or_else(|| hi_score.clone());
    Some(ScoreIniRecords { hi_score, hi_skill })
}

fn parse_score_ini_section(
    file: Option<&HashMap<String, String>>,
    drums: &HashMap<String, String>,
) -> Option<PerChartScore> {
    let mut score = PerChartScore {
        chart_hash: file
            .and_then(|file| file.get("Hash"))
            .cloned()
            .unwrap_or_default(),
        section_hash: drums.get("Hash").cloned().unwrap_or_default(),
        play_count_drums: file
            .map(|file| get_u32(file, "PlayCountDrums"))
            .unwrap_or(0),
        clear_count_drums: file
            .map(|file| get_u32(file, "ClearCountDrums"))
            .unwrap_or(0),
        history_count: file.map(|file| get_u32(file, "HistoryCount")).unwrap_or(0),
        history: [
            file.and_then(|file| file.get("History0"))
                .cloned()
                .unwrap_or_default(),
            file.and_then(|file| file.get("History1"))
                .cloned()
                .unwrap_or_default(),
            file.and_then(|file| file.get("History2"))
                .cloned()
                .unwrap_or_default(),
            file.and_then(|file| file.get("History3"))
                .cloned()
                .unwrap_or_default(),
            file.and_then(|file| file.get("History4"))
                .cloned()
                .unwrap_or_default(),
        ],
        date_time: drums.get("DateTime").cloned().unwrap_or_default(),
        progress: drums.get("Progress").cloned().unwrap_or_default(),
        score: get_u32(drums, "Score"),
        play_skill: get_f32(drums, "PlaySkill"),
        skill: get_f32(drums, "Skill"),
        perfect: get_u32(drums, "Perfect"),
        great: get_u32(drums, "Great"),
        good: get_u32(drums, "Good"),
        poor: get_u32(drums, "Poor"),
        miss: get_u32(drums, "Miss"),
        max_combo: get_u32(drums, "MaxCombo"),
        total_chips: get_u32(drums, "TotalChips"),
        auto_play: drums
            .get("AutoPlay")
            .cloned()
            .unwrap_or_else(|| "000000000000000000000000000".into()),
        play_speed_num: drums
            .get("PlaySpeed")
            .and_then(|value| value.split('/').next())
            .and_then(|num| num.parse().ok())
            .unwrap_or(20),
        play_speed_den: drums
            .get("PlaySpeed")
            .and_then(|value| value.split('/').nth(1))
            .and_then(|den| den.parse().ok())
            .unwrap_or(20),
        use_keyboard: get_bool(drums, "UseKeyboard"),
        use_midi_in: get_bool(drums, "UseMIDIIN"),
        use_joypad: get_bool(drums, "UseJoypad"),
        use_mouse: get_bool(drums, "UseMouse"),
        hit_sound_priority_hh: get_hit_sound_priority(drums, "HitSoundPriorityHH"),
        hit_sound_priority_ft: get_hit_sound_priority(drums, "HitSoundPriorityFT"),
        hit_sound_priority_cy: get_hit_sound_priority(drums, "HitSoundPriorityCY"),
        cleared: file.is_some_and(|file| get_u32(file, "ClearCountDrums") > 0),
        rank: file
            .and_then(|file| file.get("BestRankDrums"))
            .map(|rank| rank_name(rank.parse::<i32>().unwrap_or(99)).to_string())
            .unwrap_or_else(|| "UNKNOWN".into()),
        rank_guitar: file
            .and_then(|file| file.get("BestRankGuitar"))
            .and_then(|raw| raw.parse::<i32>().ok())
            .filter(|&v| (0..=6).contains(&v))
            .map(rank_name)
            .unwrap_or("")
            .to_string(),
        rank_bass: file
            .and_then(|file| file.get("BestRankBass"))
            .and_then(|raw| raw.parse::<i32>().ok())
            .filter(|&v| (0..=6).contains(&v))
            .map(rank_name)
            .unwrap_or("")
            .to_string(),
    };
    if score.section_hash.is_empty() {
        score.section_hash = compute_performance_section_md5(&score, &score.date_time);
    }
    Some(score)
}

fn parse_sections(text: &str) -> HashMap<String, HashMap<String, String>> {
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current = name.trim().to_string();
            sections.entry(current.clone()).or_default();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        sections
            .entry(current.clone())
            .or_default()
            .insert(key.trim().to_string(), value.trim().to_string());
    }
    sections
}

fn render_score_ini(score: &PerChartScore) -> String {
    render_score_ini_parts(score, score, score, score)
}

fn render_score_ini_parts(
    file_score: &PerChartScore,
    hi_score: &PerChartScore,
    hi_skill: &PerChartScore,
    last_play: &PerChartScore,
) -> String {
    let rank = rank_value(&hi_score.rank);
    let mut file_score = file_score.clone();
    let mut hi_score = hi_score.clone();
    let mut hi_skill = hi_skill.clone();
    let mut last_play = last_play.clone();
    hi_score.section_hash = compute_performance_section_md5(&hi_score, &hi_score.date_time);
    hi_skill.section_hash = compute_performance_section_md5(&hi_skill, &hi_skill.date_time);
    last_play.section_hash = compute_performance_section_md5(&last_play, &last_play.date_time);
    file_score.history_count = file_score.history_count.max(hi_score.history_count);
    let mut text = format!(
        "[File]\nTitle=\nName=\nHash={chart_hash}\nPlayCountDrums={play_count}\nPlayCountGuitars=0\nPlayCountBass=0\nClearCountDrums={clear_count}\nClearCountGuitars=0\nClearCountBass=0\nBestRankDrums={rank}\nBestRankGuitar=99\nBestRankBass=99\nHistoryCount={history_count}\nHistory0={history0}\nHistory1={history1}\nHistory2={history2}\nHistory3={history3}\nHistory4={history4}\nBGMAdjust=0\n\n",
        chart_hash = file_score.chart_hash,
        play_count = file_score.play_count_drums,
        clear_count = file_score.clear_count_drums,
        history_count = file_score.history_count,
        history0 = file_score.history[0],
        history1 = file_score.history[1],
        history2 = file_score.history[2],
        history3 = file_score.history[3],
        history4 = file_score.history[4],
    );
    for section in [
        "HiScore.Drums",
        "HiSkill.Drums",
        "HiScore.Guitar",
        "HiSkill.Guitar",
        "HiScore.Bass",
        "HiSkill.Bass",
        "LastPlay.Drums",
        "LastPlay.Guitar",
        "LastPlay.Bass",
    ] {
        if section == "HiScore.Drums" {
            render_score_section(&mut text, section, &hi_score);
        } else if section == "HiSkill.Drums" {
            render_score_section(&mut text, section, &hi_skill);
        } else if section == "LastPlay.Drums" {
            render_score_section(&mut text, section, &last_play);
        } else {
            render_empty_score_section(&mut text, section);
        }
    }
    text
}

fn render_score_section(text: &mut String, section: &str, score: &PerChartScore) {
    text.push_str(&format!(
        "[{section}]\nScore={score_value}\nPlaySkill={play_skill}\nSkill={skill}\nPerfect={perfect}\nGreat={great}\nGood={good}\nPoor={poor}\nMiss={miss}\nMaxCombo={max_combo}\nTotalChips={total_chips}\nAutoPlay={auto_play}\nRisky=0\nSuddenDrums=0\nSuddenGuitar=0\nSuddenBass=0\nHiddenDrums=0\nHiddenGuitar=0\nHiddenBass=0\nReverseDrums=0\nReverseGuitar=0\nReverseBass=0\nTightDrums=0\nRandomGuitar=0\nRandomBass=0\nLightGuitar=0\nLightBass=0\nLeftGuitar=0\nLeftBass=0\nDark=0\nScrollSpeedDrums=1\nScrollSpeedGuitar=1\nScrollSpeedBass=1\nPlaySpeed={play_speed}\nHHGroup=0\nFTGroup=0\nCYGroup=0\nBDGroup=0\nHitSoundPriorityHH={hit_sound_priority_hh}\nHitSoundPriorityFT={hit_sound_priority_ft}\nHitSoundPriorityCY={hit_sound_priority_cy}\nGuitar=0\nDrums=1\nStageFailed=1\nDamageLevel=0\nUseKeyboard={use_keyboard}\nUseMIDIIN={use_midi_in}\nUseJoypad={use_joypad}\nUseMouse={use_mouse}\nPrimaryPerfectRange=34\nPrimaryGreatRange=67\nPrimaryGoodRange=84\nPrimaryPoorRange=117\nSecondaryPerfectRange=34\nSecondaryGreatRange=67\nSecondaryGoodRange=84\nSecondaryPoorRange=117\nDTXManiaVersion=dtxpt\nDateTime={date_time}\nProgress={progress}\nHash={hash}\n\n",
        score_value = score.score,
        play_skill = score.play_skill,
        skill = score.skill,
        perfect = score.perfect,
        great = score.great,
        good = score.good,
        poor = score.poor,
        miss = score.miss,
        max_combo = score.max_combo,
        play_speed = score.play_speed_formatted(),
        total_chips = score.total_chips,
        auto_play = score.auto_play,
        use_keyboard = bool_int(score.use_keyboard),
        use_midi_in = bool_int(score.use_midi_in),
        use_joypad = bool_int(score.use_joypad),
        use_mouse = bool_int(score.use_mouse),
        hit_sound_priority_hh = hit_sound_priority_value(score.hit_sound_priority_hh),
        hit_sound_priority_ft = hit_sound_priority_value(score.hit_sound_priority_ft),
        hit_sound_priority_cy = hit_sound_priority_value(score.hit_sound_priority_cy),
        date_time = score.date_time,
        progress = progress_string(score),
        hash = score.section_hash,
    ));
}

fn render_empty_score_section(text: &mut String, section: &str) {
    render_score_section(
        text,
        section,
        &PerChartScore {
            chart_hash: String::new(),
            section_hash: String::new(),
            play_count_drums: 0,
            clear_count_drums: 0,
            history_count: 0,
            history: Default::default(),
            date_time: String::new(),
            progress: String::new(),
            score: 0,
            play_skill: 0.0,
            skill: 0.0,
            perfect: 0,
            great: 0,
            good: 0,
            poor: 0,
            miss: 0,
            max_combo: 0,
            total_chips: 0,
            auto_play: "000000000000000000000000000".into(),
            play_speed_num: 20,
            play_speed_den: 20,
            use_keyboard: false,
            use_midi_in: false,
            use_joypad: false,
            use_mouse: false,
            hit_sound_priority_hh: HitSoundPriority::ChipOverPad,
            hit_sound_priority_ft: HitSoundPriority::ChipOverPad,
            hit_sound_priority_cy: HitSoundPriority::ChipOverPad,
            cleared: false,
            rank: "UNKNOWN".into(),
            rank_guitar: String::new(),
            rank_bass: String::new(),
        },
    );
}

fn autoplay_string(auto_lanes: &std::collections::BTreeSet<DrumLane>) -> String {
    let mut chars = vec!['0'; 27];
    for (index, lane) in [
        DrumLane::Lc,
        DrumLane::Hh,
        DrumLane::Sd,
        DrumLane::Bd,
        DrumLane::Ht,
        DrumLane::Lt,
        DrumLane::Ft,
        DrumLane::Cy,
        DrumLane::Lp,
        DrumLane::Rd,
    ]
    .iter()
    .enumerate()
    {
        if auto_lanes.contains(lane) {
            chars[index] = '1';
        }
    }
    chars.into_iter().collect()
}

fn copy_history_with_new_line(
    existing: &PerChartScore,
    current: &mut PerChartScore,
    result: &RunResult,
) {
    current.play_count_drums = existing.play_count_drums.saturating_add(1).max(1);
    current.clear_count_drums = existing.clear_count_drums + u32::from(result.cleared);
    let next_count = existing.history_count.saturating_add(1).max(1);
    current.history_count = next_count;
    current.history[0] = format!(
        "{next_count}.{} {}",
        bocud_history_date(),
        history_result_text(result)
    );
    for index in 1..current.history.len() {
        current.history[index] = existing.history[index - 1].clone();
    }
}

fn history_result_text(result: &RunResult) -> String {
    if result.failed {
        "Stage failed Drums".into()
    } else if result.rank == "UNKNOWN" {
        "Cleared (No chips)".into()
    } else {
        format!("Cleared Drums ({}:{:.2})", result.rank, result.play_skill)
    }
}

fn bocud_datetime_now() -> String {
    Local::now().format("%Y/%-m/%-d %H:%M:%S").to_string()
}

fn bocud_history_date() -> String {
    let now = Local::now();
    format!("{:02}/{}/{}", now.year() % 100, now.month(), now.day())
}

fn score_beats(candidate: &PerChartScore, current: &PerChartScore) -> bool {
    candidate.score > current.score
        || (candidate.score == current.score
            && candidate.to_best_score().accuracy > current.to_best_score().accuracy)
        || (candidate.score == current.score
            && (candidate.to_best_score().accuracy - current.to_best_score().accuracy).abs()
                < f32::EPSILON
            && candidate.max_combo > current.max_combo)
}

fn get_u32(section: &HashMap<String, String>, key: &str) -> u32 {
    section.get(key).and_then(|v| v.parse().ok()).unwrap_or(0)
}

fn get_f32(section: &HashMap<String, String>, key: &str) -> f32 {
    section.get(key).and_then(|v| v.parse().ok()).unwrap_or(0.0)
}

fn get_hit_sound_priority(section: &HashMap<String, String>, key: &str) -> HitSoundPriority {
    match get_u32(section, key) {
        1 => HitSoundPriority::PadOverChip,
        _ => HitSoundPriority::ChipOverPad,
    }
}

fn hit_sound_priority_value(value: HitSoundPriority) -> i32 {
    match value {
        HitSoundPriority::ChipOverPad => 0,
        HitSoundPriority::PadOverChip => 1,
    }
}

fn get_bool(section: &HashMap<String, String>, key: &str) -> bool {
    section
        .get(key)
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn bool_int(value: bool) -> i32 {
    if value { 1 } else { 0 }
}

fn progress_string(score: &PerChartScore) -> &str {
    if score.progress.len() == 64 {
        &score.progress
    } else if score.cleared {
        "2222222222222222222222222222222222222222222222222222222222222222"
    } else if score.total_chips > 0 {
        "1111111111111111111111111111111111111111111111111111111111111111"
    } else {
        ""
    }
}

fn compute_file_md5(path: impl AsRef<Path>) -> Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(md5_hex(&bytes))
}

fn compute_performance_section_md5(score: &PerChartScore, date_time: &str) -> String {
    let mut s = String::new();
    s.push_str(&score.score.to_string());
    s.push_str(&format!("{:.6}", score.skill));
    s.push_str(&format!("{:.6}", score.play_skill));
    s.push_str(&score.perfect.to_string());
    s.push_str(&score.great.to_string());
    s.push_str(&score.good.to_string());
    s.push_str(&score.poor.to_string());
    s.push_str(&score.miss.to_string());
    s.push_str(&score.max_combo.to_string());
    s.push_str(&score.total_chips.to_string());
    s.push_str(&score.auto_play.chars().take(10).collect::<String>());
    s.push_str("000000000000000000000");
    s.push_str("1.0000001.0000001.000000");
    s.push_str("20");
    s.push_str("20");
    s.push_str("000");
    s.push_str(&hit_sound_priority_value(score.hit_sound_priority_hh).to_string());
    s.push_str(&hit_sound_priority_value(score.hit_sound_priority_ft).to_string());
    s.push_str(&hit_sound_priority_value(score.hit_sound_priority_cy).to_string());
    s.push('1');
    s.push('1');
    s.push('1');
    s.push('0');
    s.push_str(&bool_int(score.use_keyboard).to_string());
    s.push_str(&bool_int(score.use_midi_in).to_string());
    s.push_str(&bool_int(score.use_joypad).to_string());
    s.push_str(&bool_int(score.use_mouse).to_string());
    s.push_str("346784117346784117");
    s.push_str("dtxpt");
    s.push_str(date_time);
    s.push_str(progress_string(score));
    md5_hex(s.as_bytes())
}

fn md5_hex(bytes: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn rank_value(rank: &str) -> i32 {
    match rank {
        "SS" => 0,
        "S" => 1,
        "A" => 2,
        "B" => 3,
        "C" => 4,
        "D" => 5,
        "E" => 6,
        _ => 99,
    }
}

fn rank_name(rank: i32) -> &'static str {
    match rank {
        0 => "SS",
        1 => "S",
        2 => "A",
        3 => "B",
        4 => "C",
        5 => "D",
        6 => "E",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_ini_round_trips_hiscore_drums() {
        let mut score = PerChartScore {
            chart_hash: "abcdef".into(),
            section_hash: String::new(),
            play_count_drums: 1,
            clear_count_drums: 1,
            history_count: 1,
            history: [
                "1. S 987654".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            date_time: "2026/6/21 12:34:56".into(),
            progress: "2222222222222222222222222222222222222222222222222222222222222222".into(),
            score: 987_654,
            play_skill: 88.5,
            skill: 123.4,
            perfect: 100,
            great: 20,
            good: 3,
            poor: 2,
            miss: 1,
            max_combo: 111,
            total_chips: 126,
            auto_play: "000000000000000000000000000".into(),
            play_speed_num: 20,
            play_speed_den: 20,
            use_keyboard: true,
            use_midi_in: true,
            use_joypad: false,
            use_mouse: false,
            hit_sound_priority_hh: HitSoundPriority::PadOverChip,
            hit_sound_priority_ft: HitSoundPriority::ChipOverPad,
            hit_sound_priority_cy: HitSoundPriority::PadOverChip,
            cleared: true,
            rank: "S".into(),
            rank_guitar: String::new(),
            rank_bass: String::new(),
        };
        score.section_hash = compute_performance_section_md5(&score, &score.date_time);
        let parsed = parse_score_ini(&render_score_ini(&score)).unwrap();
        assert_eq!(parsed, score);
    }

    #[test]
    fn parse_preserves_separate_hiskill_drums() {
        let hi_score = PerChartScore {
            chart_hash: "abcdef".into(),
            section_hash: String::new(),
            play_count_drums: 1,
            clear_count_drums: 1,
            date_time: "2026/6/21 12:34:56".into(),
            score: 100,
            skill: 10.0,
            auto_play: "000000000000000000000000000".into(),
            rank: "A".into(),
            ..Default::default()
        };
        let hi_skill = PerChartScore {
            score: 1,
            skill: 999.0,
            ..hi_score.clone()
        };
        let records = parse_score_ini_records(&render_score_ini_parts(
            &hi_score, &hi_score, &hi_skill, &hi_score,
        ))
        .unwrap();
        assert_eq!(records.hi_score.skill, 10.0);
        assert_eq!(records.hi_skill.skill, 999.0);
    }

    #[test]
    fn to_best_score_preserves_history_for_song_select_panel() {
        let score = PerChartScore {
            history_count: 2,
            history: [
                "2026/06/22 S 123456".into(),
                "2026/06/21 A 111111".into(),
                String::new(),
                String::new(),
                String::new(),
            ],
            rank: "S".into(),
            score: 123_456,
            perfect: 10,
            great: 0,
            ..Default::default()
        };
        let best = score.to_best_score();

        assert_eq!(best.history[0], "2026/06/22 S 123456");
        assert_eq!(best.history[1], "2026/06/21 A 111111");
        assert_eq!(best.rank, "S");
    }

    #[test]
    fn score_ini_writes_bocud_default_hit_ranges() {
        let score = PerChartScore {
            chart_hash: "abcdef".into(),
            section_hash: String::new(),
            play_count_drums: 1,
            clear_count_drums: 1,
            history_count: 0,
            history: Default::default(),
            date_time: "2026/6/21 12:34:56".into(),
            progress: String::new(),
            score: 1,
            play_skill: 1.0,
            skill: 1.0,
            perfect: 1,
            great: 0,
            good: 0,
            poor: 0,
            miss: 0,
            max_combo: 1,
            total_chips: 1,
            auto_play: "000000000000000000000000000".into(),
            play_speed_num: 20,
            play_speed_den: 20,
            use_keyboard: true,
            use_midi_in: false,
            use_joypad: false,
            use_mouse: false,
            hit_sound_priority_hh: HitSoundPriority::ChipOverPad,
            hit_sound_priority_ft: HitSoundPriority::ChipOverPad,
            hit_sound_priority_cy: HitSoundPriority::ChipOverPad,
            cleared: true,
            rank: "SS".into(),
            rank_guitar: String::new(),
            rank_bass: String::new(),
        };
        let text = render_score_ini(&score);
        assert!(text.contains("PrimaryPerfectRange=34"));
        assert!(text.contains("PrimaryGreatRange=67"));
        assert!(text.contains("PrimaryGoodRange=84"));
        assert!(text.contains("PrimaryPoorRange=117"));
        assert!(text.contains("SecondaryPerfectRange=34"));
    }

    #[test]
    fn score_ini_round_trips_play_speed_num_den() {
        // BocuD stores play speed as `PlaySpeed=N/D` in the [Hi*.Drums]
        // sections. dtxpt parses it back into the num/den fields and
        // re-renders it on write.
        let mut score = PerChartScore {
            chart_hash: "abcdef".into(),
            section_hash: String::new(),
            play_count_drums: 1,
            clear_count_drums: 1,
            history_count: 0,
            history: Default::default(),
            date_time: "2026/6/22 12:34:56".into(),
            progress: String::new(),
            score: 1,
            play_skill: 1.0,
            skill: 1.0,
            perfect: 1,
            great: 0,
            good: 0,
            poor: 0,
            miss: 0,
            max_combo: 1,
            total_chips: 1,
            auto_play: "000000000000000000000000000".into(),
            play_speed_num: 18,
            play_speed_den: 20,
            use_keyboard: true,
            use_midi_in: false,
            use_joypad: false,
            use_mouse: false,
            hit_sound_priority_hh: HitSoundPriority::ChipOverPad,
            hit_sound_priority_ft: HitSoundPriority::ChipOverPad,
            hit_sound_priority_cy: HitSoundPriority::ChipOverPad,
            cleared: true,
            rank: "SS".into(),
            rank_guitar: String::new(),
            rank_bass: String::new(),
        };
        score.section_hash = compute_performance_section_md5(&score, &score.date_time);
        let text = render_score_ini(&score);
        assert!(text.contains("PlaySpeed=18/20"));
        let parsed = parse_score_ini(&text).unwrap();
        assert_eq!(parsed.play_speed_num, 18);
        assert_eq!(parsed.play_speed_den, 20);
    }

    #[test]
    fn md5_hex_matches_known_value() {
        assert_eq!(md5_hex(b"hello world"), "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn autoplay_string_uses_bocud_elane_order() {
        let mut lanes = std::collections::BTreeSet::new();
        lanes.insert(DrumLane::Bd);
        lanes.insert(DrumLane::Lp);
        lanes.insert(DrumLane::Lc);
        let autoplay = autoplay_string(&lanes);
        assert_eq!(autoplay.len(), 27);
        assert_eq!(&autoplay[..10], "1001000010");
    }

    #[test]
    fn score_ini_path_appends_bocud_suffix() {
        assert_eq!(
            score_ini_path("/tmp/song/sample.dtx"),
            PathBuf::from("/tmp/song/sample.dtx.score.ini")
        );
    }
}
