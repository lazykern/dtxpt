use bevy::prelude::*;

pub const RESTART_DOUBLE_TAP_SECS: f32 = 0.35;
pub const RESTART_HOLD_SECS: f32 = 0.50;

pub const LANE_SPEED_STEP: f32 = 0.25;
pub const TIMING_OFFSET_STEP: f32 = 0.005;
pub const MIN_LANE_SPEED: f32 = 0.5;
pub const MAX_LANE_SPEED: f32 = 8.0;
pub const HUD_PADDING: f32 = 12.0;
pub const HIT_BURST_CORE_SECS: f32 = 0.08;
pub const HIT_BURST_GLOW_SECS: f32 = 0.12;
pub const LANE_RECEPTOR_FLASH_SECS: f32 = 0.06;
pub const LANE_RECEPTOR_BASE: Color = Color::srgb(0.05, 0.05, 0.06);
pub const JUDGEMENT_SECS: f32 = 0.65;
pub const WARMUP_SECS: f32 = 2.0;
pub const TARGET_SCORE: f32 = 1_000_000.0;
pub const VISUAL_SNAP_THRESHOLD_SECS: f32 = 0.050;
pub const VISUAL_CORRECTION_GAIN: f32 = 10.0;
pub const FRAME_STATS_SMOOTHING: f32 = 0.12;
pub const SEEK_STEP_SECS: f32 = 5.0;
pub const SONG_RATE_STEP: f32 = 0.05;
pub const MIN_SONG_PLAYBACK_RATE: f32 = 0.5;
pub const MAX_SONG_PLAYBACK_RATE: f32 = 2.0;
pub const VOLUME_STEP: f32 = 0.05;
