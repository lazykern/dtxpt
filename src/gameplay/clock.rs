use bevy::prelude::*;

use crate::gameplay::constants::WARMUP_SECS;

#[derive(Resource)]
pub struct ChartClock {
    pub audio_elapsed: f32,
    pub judgement_elapsed: f32,
    /// True visual clock — advances with audio + catch-up corrections. Drives
    /// spawn windows, SE scheduling, debug HUD.
    pub visual_elapsed: f32,
    /// EMA-smoothed visual clock for rendering only. Hides catch-up jumps and
    /// rate-change pops from the user; briefly lags `visual_elapsed` by a few frames.
    pub visual_smoothed: f32,
    /// Previous frame's smoothed value. Exposed for diagnostics and for
    /// future sub-frame interpolation (a future Bevy render hook could lerp
    /// prev->current over the render frame, achieving zero-lag smoothing).
    pub prev_visual_smoothed: f32,
    /// One-frame lead of the smoothed value. `visual_smoothed + lead * song_rate`.
    /// Render systems consume this instead of `visual_smoothed` so what the user
    /// sees is approximately where the audio will be at the start of the NEXT
    /// frame, masking the 1-frame input latency inherent in render-after-update.
    pub predicted_visual: f32,
    pub audio_step_ms: f32,
    pub visual_drift_ms: f32,
    pub visual_correction_ms: f32,
}

impl Default for ChartClock {
    fn default() -> Self {
        Self {
            audio_elapsed: -WARMUP_SECS,
            judgement_elapsed: -WARMUP_SECS,
            visual_elapsed: -WARMUP_SECS,
            visual_smoothed: -WARMUP_SECS,
            prev_visual_smoothed: -WARMUP_SECS,
            predicted_visual: -WARMUP_SECS,
            audio_step_ms: 0.0,
            visual_drift_ms: 0.0,
            visual_correction_ms: 0.0,
        }
    }
}

impl ChartClock {
    pub fn reset(&mut self, timing_offset: f32) {
        *self = Self::default();
        self.judgement_elapsed = self.audio_elapsed + timing_offset;
        self.visual_elapsed = self.judgement_elapsed;
        self.visual_smoothed = self.judgement_elapsed;
        self.prev_visual_smoothed = self.judgement_elapsed;
        self.predicted_visual = self.judgement_elapsed;
    }
}

#[derive(Resource, Default)]
pub struct RenderStats {
    pub fps: f32,
    pub frame_ms: f32,
    /// Unsmoothed last-frame duration for diagnostics.
    pub raw_frame_ms: f32,
}
