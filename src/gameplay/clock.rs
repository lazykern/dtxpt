use bevy::prelude::*;

use crate::gameplay::constants::WARMUP_SECS;

#[derive(Resource)]
pub struct ChartClock {
    pub audio_elapsed: f32,
    pub judgement_elapsed: f32,
    /// True visual clock — advances with audio + catch-up corrections. Drives
    /// spawn windows, SE scheduling, debug HUD.
    pub visual_elapsed: f32,
    /// Previous fixed-tick value of `visual_elapsed`. Saved at the start of
    /// each fixed tick (in `sync_elapsed_from_audio`) before advancing.
    /// Render systems use this together with `visual_elapsed` and
    /// `Time<Fixed>::overstep_fraction()` to produce sub-frame motion.
    pub prev_visual_elapsed: f32,
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
            prev_visual_elapsed: -WARMUP_SECS,
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
        self.prev_visual_elapsed = self.judgement_elapsed;
    }
}

#[derive(Resource, Default)]
pub struct RenderStats {
    pub fps: f32,
    pub frame_ms: f32,
    /// Unsmoothed last-frame duration for diagnostics.
    pub raw_frame_ms: f32,
}
