use bevy::prelude::*;

use dtxpt::chart::Chart;

use crate::app::state::AppState;
use crate::gameplay::constants::TARGET_SCORE;
use crate::gameplay::gauge::GAUGE_CLEAR;
use crate::gameplay::{RunResult, RunState, SelectedChartPath};
use crate::persistence::{BestScore, ScoreStore, save_score_store};

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

pub fn compute_rank(accuracy: f32, failed: bool, full_combo: bool, all_perfect: bool) -> String {
    if failed {
        return "FAIL".into();
    }
    if all_perfect {
        return "SS".into();
    }
    if accuracy >= 99.0 && full_combo {
        return "S".into();
    }
    if accuracy >= 95.0 {
        return "A".into();
    }
    if accuracy >= 90.0 {
        return "B".into();
    }
    if accuracy >= 80.0 {
        return "C".into();
    }
    if accuracy >= 70.0 {
        return "D".into();
    }
    "E".into()
}

pub(crate) fn finish_to_result(
    chart: Res<Chart>,
    run: Res<RunState>,
    selected: Res<SelectedChartPath>,
    mut scores: ResMut<ScoreStore>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !run.finished {
        return;
    }

    let judged = judged_count(&run);
    let accuracy = accuracy_pct(&run);
    let full_combo = run.miss == 0 && run.poor == 0;
    let all_perfect = judged > 0 && run.perfect == judged;
    let cleared = !run.failed && run.gauge >= GAUGE_CLEAR;
    let rank = compute_rank(accuracy, run.failed, full_combo, all_perfect);
    let result = RunResult {
        title: chart.title.clone(),
        source: chart.source.clone(),
        chart_path: selected.0.clone(),
        score: run.score.round().clamp(0.0, TARGET_SCORE) as u32,
        accuracy,
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
}
