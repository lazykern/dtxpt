use bevy::prelude::*;

use crate::input::lanes::DTX_TICKS_PER_MEASURE;

#[derive(Resource, Clone)]
pub struct ChartTiming {
    pub base_bpm: f32,
    pub tempo_events: Vec<(u32, f32)>,
    pub end_tick: u32,
}

impl ChartTiming {
    pub fn new(base_bpm: f32, mut tempo_events: Vec<(u32, f32)>, end_tick: u32) -> Self {
        tempo_events.sort_by(|a, b| a.0.cmp(&b.0));
        Self {
            base_bpm,
            tempo_events,
            end_tick: end_tick.max(DTX_TICKS_PER_MEASURE),
        }
    }

    pub fn time_at_tick(&self, tick: u32) -> f32 {
        let mut current_tick = 0;
        let mut current_time = 0.0;
        let mut bpm = self.base_bpm;
        let mut bar_len = 1.0;

        for (event_tick, value) in self.tempo_events.iter().copied() {
            if event_tick > tick {
                break;
            }
            current_time += ticks_to_secs(event_tick - current_tick, bpm, bar_len);
            current_tick = event_tick;
            if value < 0.0 {
                bar_len = -value;
            } else {
                bpm = value;
            }
        }

        current_time + ticks_to_secs(tick - current_tick, bpm, bar_len)
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
    fn clamp_chart_time_respects_warmup_and_end() {
        const WARMUP: f32 = 2.0;
        let timing = ChartTiming::new(120.0, Vec::new(), 384 * 4);
        assert_eq!(clamp_chart_time(&timing, -10.0, WARMUP), -WARMUP);
        assert!(clamp_chart_time(&timing, 999.0, WARMUP) <= timing.song_end_time());
    }
}
