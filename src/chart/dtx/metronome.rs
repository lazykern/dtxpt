use crate::chart::model::MetronomeBeat;
use crate::chart::timing::ChartTiming;
use crate::input::lanes::DTX_TICKS_PER_MEASURE;

pub fn build_metronome_beats(timing: &ChartTiming, end_tick: u32) -> Vec<MetronomeBeat> {
    let end_of_song = end_tick - (end_tick % DTX_TICKS_PER_MEASURE) + DTX_TICKS_PER_MEASURE;
    let bar_lengths = timing
        .tempo_events
        .iter()
        .filter_map(|(tick, value)| (*value < 0.0).then_some((*tick, -*value)))
        .collect::<Vec<_>>();
    let mut barlength = 1.0f32;
    let mut barlength_idx = 0usize;
    let mut beat_ticks = Vec::new();

    for tick384 in (0..=end_of_song).step_by(DTX_TICKS_PER_MEASURE as usize) {
        while barlength_idx < bar_lengths.len() && bar_lengths[barlength_idx].0 <= tick384 {
            barlength = bar_lengths[barlength_idx].1;
            barlength_idx += 1;
        }
        beat_ticks.push((tick384, true));
        for i in 1..100 {
            let tick_beat = ((384 * i) as f32 / (4.0 * barlength)) as u32;
            if tick_beat >= DTX_TICKS_PER_MEASURE {
                break;
            }
            beat_ticks.push((tick384 + tick_beat, false));
        }
    }

    beat_ticks
        .into_iter()
        .map(|(tick, downbeat)| MetronomeBeat {
            time: timing.time_at_tick(tick),
            downbeat,
            fired: false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::timing::ChartTiming;

    #[test]
    fn metronome_warmup_blocks_before_chart_zero() {
        let timing = ChartTiming::new(150.0, Vec::new(), 384 * 4);
        let beats = build_metronome_beats(&timing, 384 * 4);
        assert!(beats.iter().all(|b| b.time >= 0.0));

        let mut beats = beats;
        let mut fired_during_warmup = 0usize;
        for frame in 0..125 {
            let visual = -2.0 + frame as f32 * 0.016;
            let audio = visual;
            if audio >= 0.0 && visual >= 0.0 {
                break;
            }
            for beat in beats.iter_mut() {
                if !beat.fired && visual >= beat.time {
                    fired_during_warmup += 1;
                    beat.fired = true;
                }
            }
        }
        assert_eq!(
            fired_during_warmup, 0,
            "metronome must not fire while chart time is still negative"
        );
    }
}
