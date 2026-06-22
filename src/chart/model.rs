use std::collections::BTreeMap;

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
    pub skill_level: f32,
    /// Drum-mode note slice (10 lanes, indices 0..=10). Existing
    /// drum gameplay reads from this. Guitar/Bass charts have an
    /// empty `notes` vec but populated `guitar_notes` / `bass_notes`.
    pub notes: Vec<ChartNote>,
    /// Guitar-mode note slice (5 visible lanes, plus open + control).
    /// Empty unless the DTX file has guitar channels (0x20..0x27,
    /// 0x31..0x38, 0x93..0x9A, 0x9B..0xA2, long-note 0x2C=44, wailing
    /// 0x28..0x2C, NoChip 0xBA etc.). Populated by the parser when
    /// guitar channels are encountered.
    pub guitar_notes: Vec<ChartNote>,
    /// Bass-mode note slice (4 visible lanes). Populated when bass
    /// channels are encountered.
    pub bass_notes: Vec<ChartNote>,
    /// Per-chart guitar long notes (start + end pair). Distinct from
    /// `guitar_notes` because long notes span a time range. Phase
    /// C-Guitar.2.
    pub guitar_long_notes: Vec<LongNote>,
    /// Per-chart bass long notes.
    pub bass_long_notes: Vec<LongNote>,
    pub empty_hit_events: Vec<EmptyHitEvent>,
    pub metronome_beats: Vec<MetronomeBeat>,
    pub scheduled_audio: Vec<ScheduledAudio>,
    pub wav_info: Vec<WavInfo>,
    /// Static BGA image definitions from `#BMPxx`.
    pub bga_images: Vec<BgaImageDef>,
    /// Timed BGA image layer events (`#mmm04`, `#mmm07`, and layer 3-8 channels).
    pub bga_events: Vec<BgaEvent>,
    /// Static background image from `BACKGROUND` / `STAGEFILE`.
    pub background_image: Option<String>,
    pub chart_dir: String,
    /// `#BGAPANxx` registry, keyed by BGAPAN number (1..36*36-1). Populated
    /// during parse and consulted when a `BGALayerN` chip's integer value
    /// matches a BGAPAN number — in BocuD that attaches the pan animation
    /// to that layer chip
    /// (`references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CDTX.cs:1384`).
    pub bgapan: BTreeMap<u32, BgaPanRaw>,
}

/// Long note (sustained chip) for guitar/bass. `start_time` is when
/// the chip becomes active; `end_time` is when it should be released.
/// A successful hit at `start_time` keeps the chip "held" until
/// `end_time`; an early release is judged Poor. Phase C-Guitar.2.
#[derive(Clone, Debug)]
pub struct LongNote {
    pub start_time: f32,
    pub end_time: f32,
    pub lane: usize,
    pub channel: u32,
    pub wav_id: Option<u32>,
    pub state: NoteState,
}

impl Default for Chart {
    fn default() -> Self {
        Self {
            title: String::new(),
            source: String::new(),
            bpm: 120.0,
            skill_level: 0.0,
            notes: Vec::new(),
            guitar_notes: Vec::new(),
            bass_notes: Vec::new(),
            guitar_long_notes: Vec::new(),
            bass_long_notes: Vec::new(),
            empty_hit_events: Vec::new(),
            metronome_beats: Vec::new(),
            scheduled_audio: Vec::new(),
            wav_info: Vec::new(),
            bga_images: Vec::new(),
            bga_events: Vec::new(),
            background_image: None,
            chart_dir: String::new(),
            bgapan: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BgaImageDef {
    pub id: u32,
    pub filename: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BgaEvent {
    pub time: f32,
    pub layer: u8,
    pub bmp_id: u32,
    /// BGAPAN crop/pan/size animation, if this layer chip referenced a
    /// `#BGAPANxx` directive. Mirrors DTXManiaNX-BocuD's
    /// `CChip.eBGA種別 == EBGAType.BGAPAN` path
    /// (`references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CDTX.cs:1389`).
    pub bgapan: Option<BgaPan>,
    /// True for `BGALayerN_Swap` channels; the chip references a BMPtex
    /// (`BMPTEX`) by id rather than a BMP. Semantically the renderer
    /// treats it identically, but the parser uses the marker to validate
    /// that the referenced image is actually a `BMPTEX` in BocuD.
    /// (`references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CDTX.cs:1457`).
    pub swap: bool,
}

/// BGAPAN crop/pan/size animation parameters. `transition_seconds` is the
/// final motion duration in seconds after the parser resolves the `ct`
/// tick delta via the chart timing model.
/// Mirrors `CDTX.CBGAPAN` (`references/DTXmaniaNX-BocuD/DTXMania/Score,Song/CDTX.cs:112`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BgaPan {
    pub src_start: BgaRect,
    pub src_end: BgaRect,
    pub dst_start: BgaRect,
    pub dst_end: BgaRect,
    pub transition_seconds: f32,
}

/// Raw BGAPAN parameters as parsed from the directive, before the timing
/// model resolves `transition_ticks` to seconds. Stored on `Chart.bgapan`
/// during parsing; `BgaEvent.bgapan` holds the resolved form once a layer
/// chip references it.
#[derive(Clone, Copy, Debug)]
pub struct BgaPanRaw {
    pub src_start: BgaRect,
    pub src_end: BgaRect,
    pub dst_start: BgaRect,
    pub dst_end: BgaRect,
    pub transition_ticks: i32,
}

impl BgaPanRaw {
    pub fn resolve(self, timing: &crate::chart::timing::ChartTiming) -> BgaPan {
        let transition_seconds = if self.transition_ticks <= 0 {
            0.0
        } else {
            let ct = self.transition_ticks as u32;
            timing.time_at_tick(ct)
        };
        BgaPan {
            src_start: self.src_start,
            src_end: self.src_end,
            dst_start: self.dst_start,
            dst_end: self.dst_end,
            transition_seconds,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BgaRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
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
    /// True when this note was hit by autoplay (not by player input).
    /// Used to display the "AUTO" judgement string and to filter hit counts
    /// (e.g. "100% on manual" vs "100% with 3 autoplay lanes"). Mirrors
    /// DTXMania's `CChip.bIsAutoPlayed` (CStagePerfCommonScreen.cs:1433).
    pub autoplayed: bool,
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

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum NoteState {
    #[default]
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

pub fn active_empty_hit_for_lane(
    events: &[EmptyHitEvent],
    lane: usize,
    elapsed: f32,
) -> Option<&EmptyHitEvent> {
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
        note.autoplayed = false;
    }
}

pub fn reconcile_notes_for_seek(notes: &mut [ChartNote], target: f32) {
    for note in notes {
        if note.time > target {
            note.state = NoteState::Pending;
            note.autoplayed = false;
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
                autoplayed: false,
            },
            ChartNote {
                time: 10.0,
                lane: 0,
                channel: 0x13,
                wav_id: None,
                state: NoteState::Pending,
                autoplayed: false,
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
            autoplayed: false,
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
            autoplayed: false,
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
                autoplayed: false,
            },
            ChartNote {
                time: 2.0,
                lane: 0,
                channel: 0x13,
                wav_id: None,
                state: NoteState::Missed,
                autoplayed: false,
            },
        ];
        assert!(chart_notes_complete(&notes));
        let pending = vec![ChartNote {
            time: 3.0,
            lane: 0,
            channel: 0x13,
            wav_id: None,
            state: NoteState::Pending,
            autoplayed: false,
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
            skill_level: 0.0,
            notes: Vec::new(),
            guitar_notes: Vec::new(),
            bass_notes: Vec::new(),
            guitar_long_notes: Vec::new(),
            bass_long_notes: Vec::new(),
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
            bga_images: Vec::new(),
            bga_events: Vec::new(),
            background_image: None,
            chart_dir: String::new(),
            bgapan: BTreeMap::new(),
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
