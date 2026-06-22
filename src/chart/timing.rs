use bevy::prelude::*;

use crate::input::lanes::DTX_TICKS_PER_MEASURE;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChipPlayTimeComputeMode {
    Original,
    #[default]
    Accurate,
}

impl ChipPlayTimeComputeMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Original => "Original",
            Self::Accurate => "Accurate",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Original => Self::Accurate,
            Self::Accurate => Self::Original,
        }
    }
}

#[derive(Resource, Clone)]
pub struct ChartTiming {
    pub base_bpm: f32,
    pub tempo_events: Vec<(u32, f32)>,
    pub end_tick: u32,
    pub chip_play_time_compute_mode: ChipPlayTimeComputeMode,
}

impl ChartTiming {
    pub fn new(base_bpm: f32, tempo_events: Vec<(u32, f32)>, end_tick: u32) -> Self {
        Self::with_compute_mode(
            base_bpm,
            tempo_events,
            end_tick,
            ChipPlayTimeComputeMode::default(),
        )
    }

    pub fn with_compute_mode(
        base_bpm: f32,
        mut tempo_events: Vec<(u32, f32)>,
        end_tick: u32,
        chip_play_time_compute_mode: ChipPlayTimeComputeMode,
    ) -> Self {
        tempo_events.sort_by_key(|event| event.0);
        Self {
            base_bpm,
            tempo_events,
            end_tick: end_tick.max(DTX_TICKS_PER_MEASURE),
            chip_play_time_compute_mode,
        }
    }

    pub fn time_at_tick(&self, tick: u32) -> f32 {
        let mut current_tick = 0;
        let mut current_time_ms = 0.0;
        let mut bpm = self.base_bpm;
        let mut bar_len = 1.0;

        for (event_tick, value) in self.tempo_events.iter().copied() {
            if event_tick > tick {
                break;
            }
            current_time_ms = self.compute_play_time_raw_ms(
                current_time_ms,
                event_tick - current_tick,
                bpm,
                bar_len,
            );
            current_tick = event_tick;
            if value < 0.0 {
                bar_len = -value;
            } else {
                bpm = value;
            }
        }

        self.convert_play_time_ms(self.compute_play_time_raw_ms(
            current_time_ms,
            tick - current_tick,
            bpm,
            bar_len,
        )) / 1000.0
    }

    fn compute_play_time_raw_ms(&self, start_ms: f32, ticks: u32, bpm: f32, bar_len: f32) -> f32 {
        let delta_ms = 625.0 * ticks as f32 * bar_len / bpm;
        match self.chip_play_time_compute_mode {
            ChipPlayTimeComputeMode::Original => (start_ms + delta_ms.trunc()).floor(),
            ChipPlayTimeComputeMode::Accurate => start_ms + delta_ms,
        }
    }

    fn convert_play_time_ms(&self, time_ms: f32) -> f32 {
        match self.chip_play_time_compute_mode {
            ChipPlayTimeComputeMode::Original => time_ms.trunc(),
            ChipPlayTimeComputeMode::Accurate => time_ms.round(),
        }
    }

    pub fn tick_at_time(&self, time: f32) -> u32 {
        if time <= 0.0 {
            return 0;
        }
        let mut lo = 0u32;
        let mut hi = self.end_tick;
        while lo < hi {
            let mid = lo + (hi - lo).div_ceil(2);
            if self.time_at_tick(mid) <= time {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        lo
    }

    pub fn song_end_time(&self) -> f32 {
        self.time_at_tick(self.end_tick)
    }

    pub fn time_at_measure(&self, measure: u32) -> f32 {
        self.time_at_tick(measure.saturating_mul(DTX_TICKS_PER_MEASURE))
    }

    pub fn measure_at_time(&self, time: f32) -> u32 {
        self.tick_at_time(time) / DTX_TICKS_PER_MEASURE
    }
}

pub fn ticks_to_secs(ticks: u32, bpm: f32, bar_len: f32) -> f32 {
    (625.0 * ticks as f32 * bar_len / bpm) / 1000.0
}

pub fn clamp_chart_time(timing: &ChartTiming, time: f32, warmup_secs: f32) -> f32 {
    time.clamp(-warmup_secs, timing.song_end_time())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_timing_tick_roundtrip() {
        let timing = ChartTiming::new(120.0, vec![(384, 150.0), (768, -2.0)], 384 * 8);
        for tick in [0, 96, 384, 400, 768, 2000] {
            let time = timing.time_at_tick(tick);
            let back = timing.tick_at_time(time);
            assert!(
                (back as i64 - tick as i64).abs() <= 1,
                "tick {tick} -> time {time} -> {back}"
            );
        }
    }

    #[test]
    fn chip_time_mode_matches_bocud_rounding() {
        let original = ChartTiming::with_compute_mode(
            120.0,
            Vec::new(),
            384,
            ChipPlayTimeComputeMode::Original,
        );
        let accurate = ChartTiming::with_compute_mode(
            120.0,
            Vec::new(),
            384,
            ChipPlayTimeComputeMode::Accurate,
        );
        assert!((original.time_at_tick(3) - 0.015).abs() < 0.0001);
        assert!((accurate.time_at_tick(3) - 0.016).abs() < 0.0001);
    }

    #[test]
    fn accurate_mode_preserves_fractional_segment_start() {
        let timing = ChartTiming::with_compute_mode(
            120.0,
            vec![(1, 120.0)],
            384,
            ChipPlayTimeComputeMode::Accurate,
        );
        assert!((timing.time_at_tick(3) - 0.016).abs() < 0.0001);
    }

    #[test]
    fn clamp_chart_time_respects_warmup_and_end() {
        const WARMUP: f32 = 2.0;
        let timing = ChartTiming::new(120.0, Vec::new(), 384 * 4);
        assert_eq!(clamp_chart_time(&timing, -10.0, WARMUP), -WARMUP);
        assert!(clamp_chart_time(&timing, 999.0, WARMUP) <= timing.song_end_time());
    }
}
