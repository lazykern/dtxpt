use bevy::prelude::*;

use dtxpt::chart::Judgement;
use dtxpt::input::bindings::PlayMode;

use crate::config::GameConfig;
use crate::gameplay::constants::*;
use crate::gameplay::gauge::GAUGE_START;

#[derive(Resource, Debug, Clone)]
pub struct SelectedChartPath(pub String);

#[derive(Resource, Debug, Clone)]
pub struct RunResult {
    pub title: String,
    pub source: String,
    pub chart_path: String,
    pub score: u32,
    pub accuracy: f32,
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
    pub play_mode: PlayMode,
    pub rank: String,
}

#[derive(Resource)]
pub struct RunState {
    pub raw_elapsed: f32,
    pub elapsed: f32,
    pub timing_offset: f32,
    pub lane_speed: f32,
    pub song_playback_rate: f32,
    pub metronome_sound: bool,
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
    pub judgement_timer: Timer,
    pub finished: bool,
    pub failed: bool,
    pub gauge: f32,
    pub play_mode: PlayMode,
}

impl RunState {
    pub fn from_config(config: &GameConfig) -> Self {
        Self {
            raw_elapsed: -WARMUP_SECS,
            elapsed: -WARMUP_SECS,
            timing_offset: config.timing_offset,
            lane_speed: config.lane_speed.clamp(MIN_LANE_SPEED, MAX_LANE_SPEED),
            song_playback_rate: config
                .song_playback_rate
                .clamp(MIN_SONG_PLAYBACK_RATE, MAX_SONG_PLAYBACK_RATE),
            metronome_sound: config.metronome_sound,
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
            last_delta_ms: 0.0,
            judgement_timer: Timer::from_seconds(0.0, TimerMode::Once),
            finished: false,
            failed: false,
            gauge: GAUGE_START,
            play_mode: config.play_mode,
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
