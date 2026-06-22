pub mod dtx;
pub mod loader;
pub mod model;
pub mod timing;

pub use loader::{load_chart_from_path, load_chart_from_path_with_compute_mode};
pub use model::{
    BgaEvent, BgaImageDef, BgaPan, BgaPanRaw, BgaRect, Chart, ChartNote, EmptyHitEvent, Judgement,
    MetronomeBeat, NoteState, ResultMedia, ScheduledAudio, ScheduledAudioKind, VideoDef,
    VideoEvent, VideoMode, WavInfo, WavRole, active_empty_hit_for_lane, chart_bgm_start_time,
    chart_notes_complete, reconcile_metronome_for_time, reconcile_notes_for_restart,
    reconcile_notes_for_seek, reconcile_scheduled_for_time, resolve_empty_hit_sound,
    should_suppress_metronome_beat,
};
pub use timing::{ChartTiming, ChipPlayTimeComputeMode, clamp_chart_time, ticks_to_secs};

pub use dtx::metronome::build_metronome_beats;
pub use dtx::{ChartBgm, parse_dtx_chart, parse_dtx_chart_with_compute_mode, resolve_chart_bgm};
