use bevy::prelude::*;

#[derive(Clone)]
pub struct MetronomeBeat {
    pub time: f32,
    pub downbeat: bool,
    pub fired: bool,
}

#[derive(Resource, Clone)]
pub struct Chart {
    pub title: String,
    pub source: String,
    pub bpm: f32,
    pub notes: Vec<ChartNote>,
    pub empty_hit_events: Vec<EmptyHitEvent>,
    pub metronome_beats: Vec<MetronomeBeat>,
    pub scheduled_audio: Vec<ScheduledAudio>,
    pub wav_info: Vec<WavInfo>,
    pub chart_dir: String,
}

#[derive(Clone)]
pub struct EmptyHitEvent {
    pub time: f32,
    pub lane: usize,
    pub channel: u32,
    pub wav_id: Option<u32>,
}

#[derive(Clone)]
pub struct ChartNote {
    pub time: f32,
    pub lane: usize,
    pub channel: u32,
    pub wav_id: Option<u32>,
    pub state: NoteState,
}

#[derive(Clone)]
pub struct ScheduledAudio {
    pub time: f32,
    pub wav_id: u32,
    pub kind: ScheduledAudioKind,
    pub fired: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduledAudioKind {
    Bgm,
    AutoSe { channel: u32 },
}

#[derive(Clone)]
pub struct WavInfo {
    pub id: u32,
    pub filename: String,
    pub volume: i32,
    pub pan: i32,
    pub role: WavRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WavRole {
    Drum,
    Bgm,
    Se,
    Guitar,
    Bass,
}

impl WavRole {
    pub const fn max_voices(self) -> usize {
        match self {
            Self::Drum => crate::input::lanes::POLYPHONIC_VOICES,
            Self::Bgm | Self::Se => 1,
            Self::Guitar | Self::Bass => {
                if crate::input::lanes::POLYPHONIC_VOICES >= 2 {
                    2
                } else {
                    1
                }
            }
        }
    }

    pub const fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Drum, _) | (_, Self::Drum) => Self::Drum,
            (Self::Guitar, _) | (_, Self::Guitar) => Self::Guitar,
            (Self::Bass, _) | (_, Self::Bass) => Self::Bass,
            (Self::Se, _) | (_, Self::Se) => Self::Se,
            (Self::Bgm, Self::Bgm) => Self::Bgm,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NoteState {
    Pending,
    Hit(Judgement),
    Missed,
    Skipped,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Judgement {
    Perfect,
    Great,
    Good,
    Poor,
    Miss,
}

impl Judgement {
    pub const PERFECT_WINDOW: f32 = 0.034;
    pub const GREAT_WINDOW: f32 = 0.067;
    pub const GOOD_WINDOW: f32 = 0.084;
    pub const POOR_WINDOW: f32 = 0.117;

    pub fn from_delta(delta: f32) -> Option<Self> {
        let abs = delta.abs();
        if abs <= Self::PERFECT_WINDOW {
            Some(Self::Perfect)
        } else if abs <= Self::GREAT_WINDOW {
            Some(Self::Great)
        } else if abs <= Self::GOOD_WINDOW {
            Some(Self::Good)
        } else if abs <= Self::POOR_WINDOW {
            Some(Self::Poor)
        } else {
            None
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Perfect => "PERFECT",
            Self::Great => "GREAT",
            Self::Good => "GOOD",
            Self::Poor => "POOR",
            Self::Miss => "MISS",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Perfect => Color::srgb(0.45, 0.95, 1.00),
            Self::Great => Color::srgb(0.45, 1.00, 0.45),
            Self::Good => Color::srgb(1.00, 0.90, 0.35),
            Self::Poor => Color::srgb(1.00, 0.50, 0.25),
            Self::Miss => Color::srgb(1.00, 0.20, 0.25),
        }
    }

    pub fn weight(self) -> f32 {
        match self {
            Self::Perfect => 1.0,
            Self::Great => 0.8,
            Self::Good => 0.5,
            Self::Poor => 0.2,
            Self::Miss => 0.0,
        }
    }

    pub fn keeps_combo(self) -> bool {
        matches!(self, Self::Perfect | Self::Great | Self::Good)
    }
}

pub fn chart_notes_complete(notes: &[ChartNote]) -> bool {
    notes
        .iter()
        .all(|note| !matches!(note.state, NoteState::Pending))
}

pub fn active_empty_hit_for_lane<'a>(
    events: &'a [EmptyHitEvent],
    lane: usize,
    elapsed: f32,
) -> Option<&'a EmptyHitEvent> {
    events
        .iter()
        .filter(|event| event.lane == lane && event.time <= elapsed)
        .max_by(|a, b| a.time.total_cmp(&b.time))
}

/// DTXMania B1 path: chart-defined empty-hit WAV for the pad, if any.
pub fn resolve_empty_hit_sound(
    events: &[EmptyHitEvent],
    lane: usize,
    search_lanes: &[usize],
    elapsed: f32,
) -> Option<(Option<u32>, u32)> {
    let mut order = vec![lane];
    for search_lane in search_lanes {
        if *search_lane != lane && !order.contains(search_lane) {
            order.push(*search_lane);
        }
    }
    for search_lane in order {
        if let Some(event) = active_empty_hit_for_lane(events, search_lane, elapsed) {
            return Some((event.wav_id, event.channel));
        }
    }
    None
}

pub fn chart_bgm_start_time(chart: &Chart) -> Option<f32> {
    chart
        .scheduled_audio
        .iter()
        .find(|event| matches!(event.kind, ScheduledAudioKind::Bgm))
        .map(|event| event.time)
}

pub fn should_suppress_metronome_beat(
    beat_time: f32,
    bgm_time: Option<f32>,
    stick_se_times: &[f32],
) -> bool {
    if bgm_time.is_some_and(|t| beat_time < t) {
        return true;
    }
    stick_se_times.iter().any(|t| (t - beat_time).abs() < 0.002)
}

pub fn reconcile_notes_for_restart(notes: &mut [ChartNote]) {
    for note in notes {
        note.state = NoteState::Pending;
    }
}

pub fn reconcile_notes_for_seek(notes: &mut [ChartNote], target: f32) {
    for note in notes {
        if note.time > target {
            note.state = NoteState::Pending;
        } else if note.time <= target && matches!(note.state, NoteState::Pending) {
            note.state = NoteState::Skipped;
        }
    }
}

pub fn reconcile_scheduled_for_time(scheduled: &mut [ScheduledAudio], target: f32) {
    for event in scheduled {
        event.fired = event.time <= target;
    }
}

pub fn reconcile_metronome_for_time(beats: &mut [MetronomeBeat], target: f32) {
    for beat in beats {
        beat.fired = beat.time <= target;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::lanes::POLYPHONIC_VOICES;

    #[test]
    fn wav_role_voice_caps_match_dtxmania_style_limits() {
        assert_eq!(WavRole::Drum.max_voices(), POLYPHONIC_VOICES);
        assert_eq!(WavRole::Bgm.max_voices(), 1);
        assert_eq!(WavRole::Se.max_voices(), 1);
        assert_eq!(WavRole::Guitar.max_voices(), POLYPHONIC_VOICES.min(2));
        assert_eq!(WavRole::Bass.max_voices(), POLYPHONIC_VOICES.min(2));
    }

    #[test]
    fn drum_role_wins_when_wav_is_used_multiple_ways() {
        assert_eq!(WavRole::Se.merge(WavRole::Drum), WavRole::Drum);
        assert_eq!(WavRole::Bgm.merge(WavRole::Se), WavRole::Se);
    }

    #[test]
    fn reconcile_notes_forward_skip_marks_skipped() {
        let mut notes = vec![
            ChartNote {
                time: 1.0,
                lane: 0,
                channel: 0x13,
                wav_id: None,
                state: NoteState::Pending,
            },
            ChartNote {
                time: 10.0,
                lane: 0,
                channel: 0x13,
                wav_id: None,
                state: NoteState::Pending,
            },
        ];
        reconcile_notes_for_seek(&mut notes, 5.0);
        assert!(matches!(notes[0].state, NoteState::Skipped));
        assert!(matches!(notes[1].state, NoteState::Pending));
    }

    #[test]
    fn reconcile_notes_backward_seek_resets_judged_future() {
        let mut notes = vec![ChartNote {
            time: 10.0,
            lane: 0,
            channel: 0x13,
            wav_id: None,
            state: NoteState::Missed,
        }];
        reconcile_notes_for_seek(&mut notes, 5.0);
        assert!(matches!(notes[0].state, NoteState::Pending));
    }

    #[test]
    fn reconcile_notes_backward_seek_resets_future() {
        let mut notes = vec![ChartNote {
            time: 10.0,
            lane: 0,
            channel: 0x13,
            wav_id: None,
            state: NoteState::Hit(Judgement::Perfect),
        }];
        reconcile_notes_for_seek(&mut notes, 5.0);
        assert!(matches!(notes[0].state, NoteState::Pending));
    }

    #[test]
    fn chart_notes_complete_when_none_pending() {
        let notes = vec![
            ChartNote {
                time: 1.0,
                lane: 0,
                channel: 0x13,
                wav_id: None,
                state: NoteState::Hit(Judgement::Perfect),
            },
            ChartNote {
                time: 2.0,
                lane: 0,
                channel: 0x13,
                wav_id: None,
                state: NoteState::Missed,
            },
        ];
        assert!(chart_notes_complete(&notes));
        let pending = vec![ChartNote {
            time: 3.0,
            lane: 0,
            channel: 0x13,
            wav_id: None,
            state: NoteState::Pending,
        }];
        assert!(!chart_notes_complete(&pending));
    }

    #[test]
    fn resolve_empty_hit_uses_latest_event_at_or_before_time() {
        let events = vec![
            EmptyHitEvent {
                time: 0.0,
                lane: 0,
                channel: 0xB3,
                wav_id: Some(1),
            },
            EmptyHitEvent {
                time: 10.0,
                lane: 0,
                channel: 0xB3,
                wav_id: Some(2),
            },
        ];
        let sound = resolve_empty_hit_sound(&events, 0, &[0], 12.0).unwrap();
        assert_eq!(sound, (Some(2), 0xB3));
        let earlier = resolve_empty_hit_sound(&events, 0, &[0], 5.0).unwrap();
        assert_eq!(earlier, (Some(1), 0xB3));
    }

    #[test]
    fn metronome_suppressed_before_bgm_and_with_stick_se() {
        let chart = Chart {
            title: String::new(),
            source: String::new(),
            bpm: 120.0,
            notes: Vec::new(),
            empty_hit_events: Vec::new(),
            metronome_beats: Vec::new(),
            scheduled_audio: vec![
                ScheduledAudio {
                    time: 2.0,
                    wav_id: 1,
                    kind: ScheduledAudioKind::AutoSe { channel: 0x61 },
                    fired: false,
                },
                ScheduledAudio {
                    time: 4.0,
                    wav_id: 2,
                    kind: ScheduledAudioKind::Bgm,
                    fired: false,
                },
            ],
            wav_info: Vec::new(),
            chart_dir: String::new(),
        };
        let bgm_time = chart_bgm_start_time(&chart);
        let stick_se_times = [2.0];
        assert!(should_suppress_metronome_beat(
            1.0,
            bgm_time,
            &stick_se_times
        ));
        assert!(should_suppress_metronome_beat(
            2.0,
            bgm_time,
            &stick_se_times
        ));
        assert!(!should_suppress_metronome_beat(
            4.0,
            bgm_time,
            &stick_se_times
        ));
    }
}
