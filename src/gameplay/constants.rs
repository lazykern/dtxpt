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
/// Max visual clock catch-up per frame; avoids visible note scroll jumps.
/// 1.2 frames at 60Hz, allows symmetric lead/lag for input-echo compensation.
pub const MAX_VISUAL_CORRECTION_SECS: f32 = 0.020;
/// Ignore small backward BGM position reads vs the running chart clock.
pub const MAX_AUDIO_BACKSTEP_SECS: f32 = 0.008;
/// Sub-step size for the catch-up loop. Same as MAX_VISUAL_CORRECTION_SECS by design:
/// each sub-step can advance visual by at most this much, and the loop can do up to
/// `MAX_CATCHUP_SUB_STEPS` of them per render frame.
pub const CATCHUP_SUB_STEP_SECS: f32 = 0.020;
/// Cap on sub-step iterations per render frame. 4 × 0.020 = 80ms max catch-up per frame.
pub const MAX_CATCHUP_SUB_STEPS: i32 = 4;
/// Wall-clock CPU budget for the catch-up loop, mirroring osu!lazer's
/// `max_catchup_milliseconds`. Prevents a single hitch from monopolising the frame.
pub const MAX_CATCHUP_CPU_MS: u128 = 10;
/// EMA factor thresholds for adaptive smoothing. When |visual_elapsed - visual_smoothed|
/// exceeds the big/small thresholds, the smaller alpha (more smoothing) is used to
/// hide the jump. Below the small threshold, the larger alpha (less lag) is used.
/// - 0.7 normal frame (~7% lag, snappy)
/// - 0.5 medium step (catch-up or rate change)
/// - 0.3 big jump (heavy hitch, fully hide the teleport)
pub const VISUAL_SMOOTHING_ALPHA_NORMAL: f32 = 0.7;
pub const VISUAL_SMOOTHING_ALPHA_MEDIUM: f32 = 0.5;
pub const VISUAL_SMOOTHING_ALPHA_BIG: f32 = 0.3;
/// Jump sizes (seconds) that trigger medium/big alpha.
pub const VISUAL_SMOOTH_MEDIUM_THRESHOLD: f32 = 0.020;
pub const VISUAL_SMOOTH_BIG_THRESHOLD: f32 = 0.040;
/// Lead time (seconds) for the render-ahead prediction. The visual render
/// shows `visual_smoothed + VISUAL_PREDICT_LEAD_SECS * song_rate`, which
/// approximates the audio-clock position at the start of the next render
/// frame. Hides 1 frame of input latency at the cost of a small lead
/// that the user's `timing_offset` can compensate for.
pub const VISUAL_PREDICT_LEAD_SECS: f32 = 0.010;
pub const FRAME_STATS_SMOOTHING: f32 = 0.12;
pub const SEEK_STEP_SECS: f32 = 5.0;
pub const SONG_RATE_STEP: f32 = 0.05;
pub const MIN_SONG_PLAYBACK_RATE: f32 = 0.5;
pub const MAX_SONG_PLAYBACK_RATE: f32 = 2.0;
pub const VOLUME_STEP: f32 = 0.05;
