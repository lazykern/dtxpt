use bevy::prelude::*;
use std::collections::BTreeSet;

use dtxpt::chart::Judgement;

use crate::config::{GameConfig, HitSoundPriority};
use crate::gameplay::constants::*;
use crate::gameplay::gauge::GAUGE_START;
use crate::gameplay::mods::{ModSet, resolve_auto_lanes};

#[derive(Resource, Debug, Clone)]
pub struct SelectedChartPath(pub String);

#[derive(Resource, Debug, Clone)]
pub struct RunResult {
    pub title: String,
    pub source: String,
    pub chart_path: String,
    pub score: u32,
    pub accuracy: f32,
    pub play_skill: f32,
    pub game_skill: f32,
    pub progress: String,
    pub max_combo: u32,
    pub perfect: u32,
    pub great: u32,
    pub good: u32,
    pub poor: u32,
    pub miss: u32,
    pub full_combo: bool,
    pub gauge: f32,
    pub cleared: bool,
    pub failed: bool,
    /// True if this run was in Practice mode (unscored, no gauge fail).
    pub practice: bool,
    /// Effective auto set used for this run. Snapshot of
    /// `RunState.active_mods.auto_lanes` at finish time.
    pub auto_lanes: BTreeSet<dtxpt::input::bindings::DrumLane>,
    pub used_keyboard: bool,
    pub used_midi_in: bool,
    pub used_joypad: bool,
    pub used_mouse: bool,
    /// Play speed numerator (BocuD `nPlaySpeedNumerator`) at the time
    /// the score was committed. Used for score.ini `PlaySpeed=N/D` field.
    pub play_speed_num: u32,
    /// Play speed denominator (BocuD `nPlaySpeedDenominator`).
    pub play_speed_den: u32,
    pub hit_sound_priority_hh: HitSoundPriority,
    pub hit_sound_priority_ft: HitSoundPriority,
    pub hit_sound_priority_cy: HitSoundPriority,
    pub rank: String,
}

#[derive(Resource)]
pub struct RunState {
    pub raw_elapsed: f32,
    pub elapsed: f32,
    pub timing_offset: f32,
    pub guitar_offset: f32,
    pub bass_offset: f32,
    pub lane_speed: f32,
    pub song_playback_rate: f32,
    pub metronome_sound: bool,
    pub lp_muting: bool,
    pub drum_hit_sound: bool,
    pub pedal_lag_time_ms: i32,
    pub cymbal_free: bool,
    pub hit_sound_priority_hh: HitSoundPriority,
    pub hit_sound_priority_ft: HitSoundPriority,
    pub hit_sound_priority_cy: HitSoundPriority,
    pub hit_sound_priority_lp: HitSoundPriority,
    pub show_debug_hud: bool,
    pub started: bool,
    pub score: f32,
    pub judge_units: f32,
    pub combo: u32,
    pub max_combo: u32,
    pub perfect: u32,
    pub great: u32,
    pub good: u32,
    pub poor: u32,
    pub miss: u32,
    pub last_judgement: Judgement,
    pub last_message: String,
    pub last_delta_ms: f32,
    /// True when the last judgement was applied by autoplay. HUD uses this
    /// to render the "AUTO" string instead of the judgement's own label.
    pub last_was_auto: bool,
    pub judgement_timer: Timer,
    pub finished: bool,
    pub failed: bool,
    pub gauge: f32,
    /// Top-level mode. `true` = Practice (no gauge fail, no leaderboard).
    /// Committed at song start, not toggled mid-song.
    pub practice: bool,
    /// Effective mods for this run. `auto_lanes` is derived from
    /// `GameConfig.per_lane_auto` + `GameConfig.auto_mode` at run start
    /// via `resolve_auto_lanes`. In Normal mode this is read-only during
    /// play; in Practice it can be freely toggled.
    pub active_mods: ModSet,
    pub used_keyboard: bool,
    pub used_midi_in: bool,
    pub used_joypad: bool,
    pub used_mouse: bool,
    /// Risky mode initial count. 0 = Risky off; 1..=10 = Risky N.
    /// Snapshot of `GameConfig.gauge.risky_initial` at run start.
    pub risky_initial: u8,
    /// Risky mode remaining-misses counter. Decremented on every Miss;
    /// reaching 0 fails the run (BocuD `IsFailed = risky ? risky_times <= 0 : ...`).
    pub risky_times_remaining: u8,
    /// Damage level multiplier for Poor/Miss deltas. Snapshot of
    /// `GameConfig.gauge.damage_level` at run start.
    pub damage_level: crate::config::DamageLevel,
    /// Whether auto-played chips contribute positive gauge delta.
    /// Snapshot of `GameConfig.gauge.auto_add_gauge` at run start.
    pub auto_add_gauge: bool,
    /// Play speed numerator (BocuD `nPlaySpeedNumerator`). Snapshot of
    /// `GameConfig.play_speed_num` at run start. Used by score.ini codec
    /// (`PlaySpeed=N/D` field).
    pub play_speed_num: u32,
    /// Play speed denominator (BocuD `nPlaySpeedDenominator`). Snapshot
    /// of `GameConfig.play_speed_den` at run start.
    pub play_speed_den: u32,
}

impl RunState {
    pub fn from_config(config: &GameConfig) -> Self {
        let active_mods = ModSet {
            auto_lanes: resolve_auto_lanes(&config.per_lane_auto, config.auto_mode),
        };
        Self {
            raw_elapsed: -WARMUP_SECS,
            elapsed: -WARMUP_SECS,
            timing_offset: config.timing_offset,
            guitar_offset: config.guitar_offset,
            bass_offset: config.bass_offset,
            lane_speed: config.lane_speed.clamp(MIN_LANE_SPEED, MAX_LANE_SPEED),
            song_playback_rate: config
                .song_playback_rate
                .clamp(MIN_SONG_PLAYBACK_RATE, MAX_SONG_PLAYBACK_RATE),
            metronome_sound: config.metronome_sound,
            lp_muting: config.lp_muting,
            drum_hit_sound: config.drum_hit_sound,
            pedal_lag_time_ms: config.pedal_lag_time_ms,
            cymbal_free: config.cymbal_free,
            hit_sound_priority_hh: config.hit_sound_priority_hh,
            hit_sound_priority_ft: config.hit_sound_priority_ft,
            hit_sound_priority_cy: config.hit_sound_priority_cy,
            hit_sound_priority_lp: config.hit_sound_priority_lp,
            show_debug_hud: config.show_debug_hud,
            started: false,
            score: 0.0,
            judge_units: 0.0,
            combo: 0,
            max_combo: 0,
            perfect: 0,
            great: 0,
            good: 0,
            poor: 0,
            miss: 0,
            last_judgement: Judgement::Miss,
            last_message: "READY".into(),
            last_was_auto: false,
            last_delta_ms: 0.0,
            judgement_timer: Timer::from_seconds(0.0, TimerMode::Once),
            finished: false,
            failed: false,
            gauge: GAUGE_START,
            practice: config.practice_song_select,
            active_mods,
            used_keyboard: false,
            used_midi_in: false,
            used_joypad: false,
            used_mouse: false,
            risky_initial: config.gauge.risky_initial,
            risky_times_remaining: config.gauge.risky_initial,
            damage_level: config.gauge.damage_level,
            auto_add_gauge: config.gauge.auto_add_gauge,
            play_speed_num: config.play_speed_num,
            play_speed_den: config.play_speed_den,
        }
    }
}

impl Default for RunState {
    fn default() -> Self {
        Self::from_config(&GameConfig::default())
    }
}

pub fn gameplay_dev_hotkeys_enabled(run: Res<RunState>) -> bool {
    run.show_debug_hud
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::mods::AutoMode;
    use dtxpt::input::bindings::DrumLane;
    use std::collections::BTreeSet;

    fn cfg_with(per_lane: BTreeSet<DrumLane>, mode: AutoMode, practice: bool) -> GameConfig {
        GameConfig {
            per_lane_auto: per_lane,
            auto_mode: mode,
            practice_song_select: practice,
            ..GameConfig::default()
        }
    }

    #[test]
    fn run_state_from_config_resolves_perlane() {
        let mut per_lane = BTreeSet::new();
        per_lane.insert(DrumLane::Bd);
        per_lane.insert(DrumLane::Hh);
        let run = RunState::from_config(&cfg_with(per_lane.clone(), AutoMode::PerLane, false));
        assert_eq!(run.active_mods.auto_lanes, per_lane);
        assert!(!run.practice);
    }

    #[test]
    fn run_state_from_config_resolves_all_auto() {
        let cfg = cfg_with(BTreeSet::new(), AutoMode::AllAuto, false);
        let run = RunState::from_config(&cfg);
        assert_eq!(run.active_mods.auto_lanes.len(), DrumLane::ALL.len());
    }

    #[test]
    fn run_state_from_config_resolves_off() {
        let mut per_lane = BTreeSet::new();
        per_lane.insert(DrumLane::Hh);
        let cfg = cfg_with(per_lane.clone(), AutoMode::Off, false);
        let run = RunState::from_config(&cfg);
        assert!(run.active_mods.auto_lanes.is_empty());
        // User's per-lane config is preserved untouched.
        assert_eq!(per_lane.len(), 1);
    }

    #[test]
    fn run_state_from_config_propagates_practice() {
        let cfg = cfg_with(BTreeSet::new(), AutoMode::Off, true);
        let run = RunState::from_config(&cfg);
        assert!(run.practice);
    }

    #[test]
    fn run_state_default_picks_up_auto_lanes_from_default_config() {
        let run = RunState::from_config(&GameConfig::default());
        // Default config: per_lane_auto empty, auto_mode PerLane -> empty
        assert!(run.active_mods.auto_lanes.is_empty());
        // Default config: practice_song_select false
        assert!(!run.practice);
    }
}
