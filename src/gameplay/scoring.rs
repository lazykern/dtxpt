use bevy::prelude::*;

use dtxpt::chart::{Chart, Judgement, NoteState};
use dtxpt::input::bindings::DrumLane;

use crate::app::state::AppState;
use crate::config::{GameConfig, SkillMode};
use crate::gameplay::constants::TARGET_SCORE;
use crate::gameplay::gauge::GAUGE_CLEAR;
use crate::gameplay::{RunResult, RunState, SelectedChartPath};
use crate::persistence::{
    BestScore, ScoreStore, save_score_store, score_ini_path, write_score_ini_result,
};

pub fn judged_count(run: &RunState) -> u32 {
    run.perfect + run.great + run.good + run.poor + run.miss
}

pub fn accuracy_pct(run: &RunState) -> f32 {
    let judged = judged_count(run);
    if judged == 0 {
        0.0
    } else {
        (run.judge_units / judged as f32) * 100.0
    }
}

pub fn display_score(run: &RunState) -> u32 {
    run.score.round().clamp(0.0, TARGET_SCORE) as u32
}

pub fn compute_rank(run: &RunState, skill_mode: SkillMode) -> String {
    // BocuD quirk: when all lanes are on autoplay, every note is auto-acted,
    // so the rank is treated as perfect (SS) regardless of judgement counts.
    // Matches BocuD `tCalculateRank(nTotal, ...)` returning SS when
    // `nTotal == nAuto`.
    if run.active_mods.is_all_lanes() && judged_count(run) > 0 {
        return "SS".into();
    }
    if !(run.used_keyboard || run.used_midi_in || run.used_joypad || run.used_mouse) {
        return "UNKNOWN".into();
    }
    match skill_mode {
        SkillMode::Old => compute_rank_old(run.perfect, run.great, run.good, run.poor, run.miss),
        SkillMode::New => compute_rank_from_completion_rate(playing_skill_rate(run)),
    }
    .into()
}

pub fn compute_rank_old(perfect: u32, great: u32, good: u32, poor: u32, miss: u32) -> &'static str {
    let total = perfect + great + good + poor + miss;
    if total == 0 {
        return "UNKNOWN";
    }
    let rate = (perfect + great) as f32 / total as f32;
    if (rate - 1.0).abs() < f32::EPSILON {
        "SS"
    } else if rate >= 0.95 {
        "S"
    } else if rate >= 0.90 {
        "A"
    } else if rate >= 0.85 {
        "B"
    } else if rate >= 0.80 {
        "C"
    } else if rate >= 0.70 {
        "D"
    } else {
        "E"
    }
}

pub fn playing_skill_rate(run: &RunState) -> f32 {
    let total = judged_count(run);
    if total == 0 {
        return 0.0;
    }
    let total = total as f32;
    let perfect_rate = 100.0 * run.perfect as f32 / total;
    let great_rate = 100.0 * run.great as f32 / total;
    let combo_rate = 100.0 * run.max_combo as f32 / total;
    perfect_rate * 0.85 + great_rate * 0.35 + combo_rate * 0.15
}

pub fn playing_skill_rate_old(run: &RunState) -> f32 {
    let total = judged_count(run);
    if total == 0 {
        return 0.0;
    }
    100.0 * (run.perfect as f32 * 0.8 + run.great as f32 * 0.3 + run.max_combo as f32 * 0.2)
        / total as f32
}

pub fn play_skill(run: &RunState, skill_mode: SkillMode) -> f32 {
    let base = match skill_mode {
        SkillMode::Old => playing_skill_rate_old(run),
        SkillMode::New => playing_skill_rate(run),
    };
    base * auto_skill_revise(run)
}

pub fn game_skill(skill_level: f32, run: &RunState, skill_mode: SkillMode) -> f32 {
    if run.active_mods.auto_lanes.len() == DrumLane::ALL.len() {
        return 0.0;
    }
    match skill_mode {
        SkillMode::Old => skill_level * (play_skill(run, SkillMode::Old) / 100.0) * 0.33,
        SkillMode::New => skill_level * play_skill(run, SkillMode::New) * 0.2,
    }
}

fn auto_skill_revise(run: &RunState) -> f32 {
    let auto = &run.active_mods.auto_lanes;
    let bd = auto.contains(&DrumLane::Bd);
    let lp = auto.contains(&DrumLane::Lp);
    if (bd && !lp) || (!bd && lp) { 0.5 } else { 1.0 }
}

pub fn progress_string(chart: &Chart) -> String {
    const SECTIONS: usize = 64;
    let last_time = chart
        .notes
        .iter()
        .map(|note| note.time)
        .fold(0.0_f32, f32::max);
    if chart.notes.is_empty() || last_time <= 0.0 {
        return String::new();
    }
    let mut chip_count = [0_u32; SECTIONS];
    let mut hit_count = [0_u32; SECTIONS];
    for note in &chart.notes {
        let index = ((note.time * SECTIONS as f32 / last_time) as usize).min(SECTIONS - 1);
        chip_count[index] += 1;
        if matches!(
            note.state,
            NoteState::Hit(Judgement::Perfect | Judgement::Great | Judgement::Good)
        ) {
            hit_count[index] += 1;
        }
    }
    (0..SECTIONS)
        .map(|index| {
            if chip_count[index] == 0 {
                '3'
            } else if hit_count[index] == chip_count[index] {
                '2'
            } else {
                '1'
            }
        })
        .collect()
}

pub fn compute_rank_from_completion_rate(completion_rate: f32) -> &'static str {
    if completion_rate <= 0.0 {
        "UNKNOWN"
    } else if completion_rate >= 95.0 {
        "SS"
    } else if completion_rate >= 80.0 {
        "S"
    } else if completion_rate >= 73.0 {
        "A"
    } else if completion_rate >= 63.0 {
        "B"
    } else if completion_rate >= 53.0 {
        "C"
    } else if completion_rate >= 45.0 {
        "D"
    } else {
        "E"
    }
}

pub(crate) fn finish_to_result(
    chart: Res<Chart>,
    run: Res<RunState>,
    config: Res<GameConfig>,
    selected: Res<SelectedChartPath>,
    mut scores: ResMut<ScoreStore>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !run.finished {
        return;
    }

    let accuracy = accuracy_pct(&run);
    let full_combo = run.miss == 0 && run.poor == 0;
    let cleared = !run.failed && run.gauge >= GAUGE_CLEAR;
    let play_skill = play_skill(&run, config.skill_mode);
    let game_skill = game_skill(chart.skill_level, &run, config.skill_mode);
    let rank = compute_rank(&run, config.skill_mode);
    let result = RunResult {
        title: chart.title.clone(),
        source: chart.source.clone(),
        chart_path: selected.0.clone(),
        score: run.score.round().clamp(0.0, TARGET_SCORE) as u32,
        accuracy,
        play_skill,
        game_skill,
        progress: progress_string(&chart),
        max_combo: run.max_combo,
        perfect: run.perfect,
        great: run.great,
        good: run.good,
        poor: run.poor,
        miss: run.miss,
        full_combo,
        gauge: run.gauge,
        cleared,
        failed: run.failed,
        practice: run.practice,
        auto_lanes: run.active_mods.auto_lanes.clone(),
        used_keyboard: run.used_keyboard,
        used_midi_in: run.used_midi_in,
        used_joypad: run.used_joypad,
        used_mouse: run.used_mouse,
        play_speed_num: run.play_speed_num,
        play_speed_den: run.play_speed_den,
        hit_sound_priority_hh: run.hit_sound_priority_hh,
        hit_sound_priority_ft: run.hit_sound_priority_ft,
        hit_sound_priority_cy: run.hit_sound_priority_cy,
        rank,
    };
    // Practice runs and any run with auto lanes do not compete on the
    // "no mods" best board. The user explicitly opted into assistance
    // for this run, so the score is recorded but not compared.
    let is_new_best = !run.practice
        && run.active_mods.auto_lanes.is_empty()
        && scores
            .scores
            .get(&result.chart_path)
            .map(|best| best.beats(&result))
            .unwrap_or(true);
    if is_new_best {
        scores
            .scores
            .insert(result.chart_path.clone(), BestScore::from_result(&result));
        if let Err(err) = save_score_store(&scores) {
            warn!("failed to save scores: {err}");
        }
    }
    if config.write_score_ini
        && !run.practice
        && let Err(err) = write_score_ini_result(score_ini_path(&result.chart_path), &result)
    {
        warn!("failed to save score.ini: {err}");
    }
    commands.insert_resource(result);
    next_state.set(AppState::Result);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_run() -> RunState {
        RunState {
            perfect: 2,
            great: 1,
            good: 0,
            poor: 0,
            miss: 1,
            judge_units: 2.0 + 1.0,
            score: 12_345.6,
            combo: 3,
            max_combo: 5,
            gauge: 0.9,
            practice: false,
            active_mods: Default::default(),
            ..Default::default()
        }
    }

    #[test]
    fn judged_count_sums_counters() {
        let run = sample_run();
        assert_eq!(judged_count(&run), 4_u32);
    }

    #[test]
    fn accuracy_pct_uses_judge_units() {
        let run = sample_run();
        assert!((accuracy_pct(&run) - 75.0).abs() < f32::EPSILON);
    }

    #[test]
    fn display_score_clamps_to_target() {
        let mut run = sample_run();
        run.score = TARGET_SCORE + 500.0;
        assert_eq!(display_score(&run), TARGET_SCORE as u32);
    }

    #[test]
    fn rank_is_unknown_without_input_device() {
        let run = sample_run();
        assert_eq!(compute_rank(&run, SkillMode::New), "UNKNOWN");
    }

    #[test]
    fn rank_is_ss_when_all_lanes_are_auto() {
        // BocuD quirk: a run with every lane on autoplay always returns SS
        // even if the judgement counts would otherwise rank lower.
        let mut run = sample_run();
        run.active_mods.auto_lanes = DrumLane::ALL.iter().copied().collect();
        run.used_keyboard = true;
        // Even with most chips Poor/Miss, the rank is SS.
        run.perfect = 0;
        run.great = 0;
        run.good = 0;
        run.poor = 5;
        run.miss = 5;
        run.judge_units = 5.0;
        assert_eq!(compute_rank(&run, SkillMode::New), "SS");
        assert_eq!(compute_rank(&run, SkillMode::Old), "SS");
    }

    #[test]
    fn progress_string_uses_bocud_section_chars() {
        let chart = Chart {
            title: String::new(),
            source: String::new(),
            bpm: 120.0,
            skill_level: 0.0,
            notes: vec![
                dtxpt::chart::ChartNote {
                    time: 1.0,
                    lane: 0,
                    channel: 0x11,
                    wav_id: None,
                    state: NoteState::Hit(Judgement::Perfect),
                    autoplayed: false,
                },
                dtxpt::chart::ChartNote {
                    time: 2.0,
                    lane: 0,
                    channel: 0x11,
                    wav_id: None,
                    state: NoteState::Missed,
                    autoplayed: false,
                },
            ],
            guitar_notes: Vec::new(),
            bass_notes: Vec::new(),
            guitar_long_notes: Vec::new(),
            bass_long_notes: Vec::new(),
            empty_hit_events: Vec::new(),
            metronome_beats: Vec::new(),
            scheduled_audio: Vec::new(),
            wav_info: Vec::new(),
            bga_images: Vec::new(),
            bga_events: Vec::new(),
            background_image: None,
            chart_dir: String::new(),
            bgapan: std::collections::BTreeMap::new(),
            avi_files: Vec::new(),
            video_events: Vec::new(),
            avipan: std::collections::BTreeMap::new(),
            premovie: None,
            result_image: dtxpt::chart::ResultMedia::default(),
            result_movie: dtxpt::chart::ResultMedia::default(),
            result_sound: dtxpt::chart::ResultMedia::default(),
        };
        let progress = progress_string(&chart);
        assert_eq!(progress.len(), 64);
        assert_eq!(progress.chars().filter(|c| *c == '2').count(), 1);
        assert_eq!(progress.chars().filter(|c| *c == '1').count(), 1);
    }

    #[test]
    fn rank_old_matches_bocud_thresholds() {
        assert_eq!(compute_rank_old(10, 0, 0, 0, 0), "SS");
        assert_eq!(compute_rank_old(9, 1, 0, 0, 0), "SS");
        assert_eq!(compute_rank_old(18, 1, 1, 0, 0), "S");
        assert_eq!(compute_rank_old(9, 0, 1, 0, 0), "A");
        assert_eq!(compute_rank_old(17, 0, 3, 0, 0), "B");
        assert_eq!(compute_rank_old(16, 0, 4, 0, 0), "C");
        assert_eq!(compute_rank_old(14, 0, 6, 0, 0), "D");
        assert_eq!(compute_rank_old(13, 0, 7, 0, 0), "E");
        assert_eq!(compute_rank_old(0, 0, 0, 0, 0), "UNKNOWN");
    }

    #[test]
    fn rank_new_matches_bocud_completion_thresholds() {
        assert_eq!(compute_rank_from_completion_rate(95.0), "SS");
        assert_eq!(compute_rank_from_completion_rate(80.0), "S");
        assert_eq!(compute_rank_from_completion_rate(73.0), "A");
        assert_eq!(compute_rank_from_completion_rate(63.0), "B");
        assert_eq!(compute_rank_from_completion_rate(53.0), "C");
        assert_eq!(compute_rank_from_completion_rate(45.0), "D");
        assert_eq!(compute_rank_from_completion_rate(44.99), "E");
        assert_eq!(compute_rank_from_completion_rate(0.0), "UNKNOWN");
    }

    #[test]
    fn game_skill_matches_bocud_new_formula() {
        let mut run = sample_run();
        run.perfect = 10;
        run.great = 0;
        run.good = 0;
        run.poor = 0;
        run.miss = 0;
        run.max_combo = 10;
        assert!((game_skill(8.5, &run, SkillMode::New) - 170.0).abs() < 0.001);
    }

    #[test]
    fn old_playing_skill_matches_bocud_formula() {
        let mut run = sample_run();
        run.perfect = 8;
        run.great = 1;
        run.good = 1;
        run.poor = 0;
        run.miss = 0;
        run.max_combo = 10;
        let expected = 100.0 * (8.0 * 0.8 + 1.0 * 0.3 + 10.0 * 0.2) / 10.0;
        assert!((playing_skill_rate_old(&run) - expected).abs() < 0.001);
    }

    #[test]
    fn drum_auto_skill_revise_matches_bocud_bd_lp_subset() {
        let mut run = sample_run();
        assert_eq!(auto_skill_revise(&run), 1.0);
        run.active_mods.auto_lanes.insert(DrumLane::Bd);
        assert_eq!(auto_skill_revise(&run), 0.5);
        run.active_mods.auto_lanes.insert(DrumLane::Lp);
        assert_eq!(auto_skill_revise(&run), 1.0);
        run.active_mods.auto_lanes.remove(&DrumLane::Bd);
        assert_eq!(auto_skill_revise(&run), 0.5);
        run.active_mods.auto_lanes.insert(DrumLane::Sd);
        assert_eq!(auto_skill_revise(&run), 0.5);
    }

    #[test]
    fn playing_skill_rate_matches_bocud_new_formula() {
        let mut run = sample_run();
        run.perfect = 8;
        run.great = 1;
        run.good = 1;
        run.poor = 0;
        run.miss = 0;
        run.max_combo = 10;
        let expected = 80.0 * 0.85 + 10.0 * 0.35 + 100.0 * 0.15;
        assert!((playing_skill_rate(&run) - expected).abs() < f32::EPSILON);
    }
}
