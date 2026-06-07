use bevy::prelude::*;
use dtxpt::input::bindings::{InputBindingConfig, PlayMode, default_input_bindings};
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    pub version: u32,
    pub chart_root: String,
    #[serde(default)]
    pub last_chart_path: String,
    #[serde(default)]
    pub preferred_difficulty: String,
    pub master_volume: f64,
    pub bgm_volume: f64,
    pub drum_volume: f64,
    pub lane_speed: f32,
    pub timing_offset: f32,
    pub song_playback_rate: f32,
    pub play_mode: PlayMode,
    pub bindings: Vec<InputBindingConfig>,
    #[serde(default, alias = "lane_keys", skip_serializing)]
    pub legacy_lane_keys: Option<[String; 10]>,
    pub vsync: bool,
    pub metronome_sound: bool,
    pub show_debug_hud: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            version: 7,
            chart_root: "charts".into(),
            last_chart_path: String::new(),
            preferred_difficulty: String::new(),
            master_volume: 0.8,
            bgm_volume: 1.0,
            drum_volume: 1.0,
            lane_speed: 1.0,
            timing_offset: 0.0,
            song_playback_rate: 1.0,
            play_mode: PlayMode::Normal,
            bindings: default_input_bindings(),
            legacy_lane_keys: None,
            vsync: true,
            metronome_sound: true,
            show_debug_hud: false,
        }
    }
}
