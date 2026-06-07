use bevy::prelude::*;

use crate::gameplay::constants::WARMUP_SECS;

#[derive(Resource)]
pub struct ChartClock {
    pub audio_elapsed: f32,
    pub judgement_elapsed: f32,
    pub visual_elapsed: f32,
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
    }
}

#[derive(Resource, Default)]
pub struct RenderStats {
    pub fps: f32,
    pub frame_ms: f32,
}
