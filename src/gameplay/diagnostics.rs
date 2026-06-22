use std::sync::atomic::{AtomicU32, Ordering};

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy::winit::WinitSettings;

use dtxpt::util::diag;

use crate::app::markers::{MetronomeLineVisual, NoteVisual};
use crate::app::state::{PauseState, is_paused};
use crate::audio::{ActiveSounds, BackgroundDecodeReceiver, count_tracked_live_voices};
use crate::gameplay::clock::{ChartClock, RenderStats};
use crate::gameplay::constants::MAX_VISUAL_CORRECTION_SECS;
use crate::gameplay::run::RunState;

pub const FRAME_SPIKE_MS: f32 = 20.0;
pub const AUDIO_STEP_SPIKE_MS: f32 = 35.0;
const DIAG_SUMMARY_SECS: f32 = 5.0;

static VOICE_PREEMPT_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn record_voice_preempt() {
    VOICE_PREEMPT_COUNTER.fetch_add(1, Ordering::Relaxed);
}

pub fn take_voice_preempt_count() -> u32 {
    VOICE_PREEMPT_COUNTER.swap(0, Ordering::Relaxed)
}

#[derive(Resource, Default, Debug, Clone)]
pub struct PlaybackDiagnostics {
    pub frame_spikes: u32,
    pub peak_frame_ms: f32,
    pub audio_step_spikes: u32,
    pub visual_snaps: u32,
    pub metronome_beats: u32,
    pub voice_preempts: u32,
    pub metro_line_despawns: u32,
    pub bg_decode_merges: u32,
    pub midi_rescans: u32,
}

pub fn diag_active(run: &RunState) -> bool {
    diag::env_diag_enabled() || run.show_debug_hud
}

pub fn reset_playback_diagnostics(mut diag: ResMut<PlaybackDiagnostics>) {
    *diag = PlaybackDiagnostics::default();
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn monitor_playback_diagnostics(
    run: Res<RunState>,
    pause_state: Res<State<PauseState>>,
    clock: Res<ChartClock>,
    stats: Res<RenderStats>,
    active: Res<ActiveSounds>,
    decode: Option<Res<BackgroundDecodeReceiver>>,
    audio_instances: Res<Assets<bevy_kira_audio::prelude::AudioInstance>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    winit: Res<WinitSettings>,
    note_visuals: Query<(), With<NoteVisual>>,
    metro_lines: Query<(), With<MetronomeLineVisual>>,
    mut playback_diag: ResMut<PlaybackDiagnostics>,
    mut last_summary_at: Local<f32>,
) {
    if !diag_active(&run) || is_paused(pause_state.get()) {
        return;
    }

    playback_diag.voice_preempts += take_voice_preempt_count();
    playback_diag.midi_rescans += diag::take_midi_rescan_count();

    let raw_frame_ms = stats.raw_frame_ms;
    if raw_frame_ms > FRAME_SPIKE_MS {
        playback_diag.frame_spikes += 1;
        playback_diag.peak_frame_ms = playback_diag.peak_frame_ms.max(raw_frame_ms);
        let live_voices = count_tracked_live_voices(&active, &audio_instances);
        let pending_decode = decode.as_ref().map(|r| r.pending).unwrap_or(0);
        warn!(
            "frame spike {:.2}ms (smooth {:.2}ms) @ chart {:.3}s | audio_step {:+.1}ms drift {:+.1}ms correct {:+.1}ms | voices={live_voices} decode_pending={pending_decode}",
            raw_frame_ms,
            stats.frame_ms,
            run.elapsed,
            clock.audio_step_ms,
            clock.visual_drift_ms,
            clock.visual_correction_ms,
        );
    }

    if clock.audio_step_ms.abs() > AUDIO_STEP_SPIKE_MS
        && clock.audio_step_ms.abs() > stats.raw_frame_ms + 5.0
    {
        playback_diag.audio_step_spikes += 1;
        debug!(
            "audio clock step {:+.1}ms @ chart {:.3}s (raw {:.3}s)",
            clock.audio_step_ms, run.elapsed, clock.audio_elapsed,
        );
    }

    if clock.visual_correction_ms.abs() >= MAX_VISUAL_CORRECTION_SECS * 1000.0 * 0.95 {
        playback_diag.visual_snaps += 1;
        debug!(
            "visual catchup {:+.1}ms @ chart {:.3}s drift {:+.1}ms",
            clock.visual_correction_ms, run.elapsed, clock.visual_drift_ms,
        );
    }

    let summary_anchor = run.elapsed.max(0.0);
    if summary_anchor - *last_summary_at >= DIAG_SUMMARY_SECS {
        *last_summary_at = summary_anchor;
        let live_voices = count_tracked_live_voices(&active, &audio_instances);
        let pending_decode = decode.as_ref().map(|r| r.pending).unwrap_or(0);
        let note_count = note_visuals.iter().count();
        let metro_count = metro_lines.iter().count();
        let (focused, present_mode, winit_active) = windows
            .single()
            .map(|window| {
                (
                    window.focused.to_string(),
                    format!("{:?}", window.present_mode),
                    format!("{:?}", winit.update_mode(window.focused)),
                )
            })
            .unwrap_or_else(|_| ("n/a".to_string(), "n/a".to_string(), "n/a".to_string()));
        debug!(
            "diag summary @ {:.1}s | spikes={} peak={:.1}ms audio_steps={} visual_snaps={} metro={} preempts={} line_despawn={} decode={} midi_rescan={} notes={note_count} metro_lines={metro_count} voices={live_voices} decode_pending={pending_decode} focused={focused} present={present_mode} winit_active={winit_active} winit_focused={:?} winit_unfocused={:?}",
            summary_anchor,
            playback_diag.frame_spikes,
            playback_diag.peak_frame_ms,
            playback_diag.audio_step_spikes,
            playback_diag.visual_snaps,
            playback_diag.metronome_beats,
            playback_diag.voice_preempts,
            playback_diag.metro_line_despawns,
            playback_diag.bg_decode_merges,
            playback_diag.midi_rescans,
            winit.focused_mode,
            winit.unfocused_mode,
        );
    }
}
