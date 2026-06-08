#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};

use crate::app::markers::*;
use crate::app::state::{PauseState, is_paused};
use crate::audio::{ActiveSounds, AudioMix, BackgroundDecodeReceiver, count_tracked_live_voices};
use crate::gameplay::clock::{ChartClock, RenderStats};
use crate::gameplay::constants::FRAME_STATS_SMOOTHING;
use crate::gameplay::diagnostics::PlaybackDiagnostics;
use crate::gameplay::gauge::gauge_fill_color;
use crate::config::GameConfig;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;
use crate::gameplay::scoring::{accuracy_pct, display_score};
use dtxpt::chart::{Chart, chart_notes_complete};

#[derive(Resource, Default)]
pub(crate) struct HudDisplayCache {
    score_text: Option<String>,
    accuracy_text: Option<String>,
    combo_text: Option<String>,
    counters_text: Option<String>,
    gauge_text: Option<String>,
    gauge_fill_pct: Option<f32>,
    gauge_fill_failed: Option<bool>,
    debug_text: Option<String>,
    judgement_text: Option<String>,
    judgement_color: Option<Color>,
}

fn set_text_if_changed(text: &mut Text, cache: &mut Option<String>, new_text: String) {
    if cache.as_ref() != Some(&new_text) {
        text.0 = new_text.clone();
        *cache = Some(new_text);
    }
}

pub fn update_render_stats(time: Res<Time>, mut stats: ResMut<RenderStats>) {
    let dt = time.delta_secs();
    if dt <= f32::EPSILON {
        return;
    }

    let fps = 1.0 / dt;
    let frame_ms = dt * 1000.0;
    stats.raw_frame_ms = frame_ms;
    if stats.fps == 0.0 {
        stats.fps = fps;
        stats.frame_ms = frame_ms;
    } else {
        stats.fps += (fps - stats.fps) * FRAME_STATS_SMOOTHING;
        stats.frame_ms += (frame_ms - stats.frame_ms) * FRAME_STATS_SMOOTHING;
    }
}

pub(crate) fn sync_debug_hud_visibility(
    run: Res<RunState>,
    mut last: Local<Option<bool>>,
    mut ui: Query<&mut Visibility, With<GameplayHudDebug>>,
) {
    if last.as_ref() == Some(&run.show_debug_hud) {
        return;
    }
    *last = Some(run.show_debug_hud);

    let visibility = if run.show_debug_hud {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut vis in ui.iter_mut() {
        *vis = visibility;
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_hud(
    chart: Res<Chart>,
    run: Res<RunState>,
    config: Res<GameConfig>,
    pause_state: Res<State<PauseState>>,
    clock: Res<ChartClock>,
    mix: Res<AudioMix>,
    stats: Res<RenderStats>,
    layout: Res<PlayfieldLayout>,
    playback_diag: Res<PlaybackDiagnostics>,
    active: Res<ActiveSounds>,
    decode: Option<Res<BackgroundDecodeReceiver>>,
    audio_instances: Res<Assets<bevy_kira_audio::prelude::AudioInstance>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cache: ResMut<HudDisplayCache>,
    mut hud_text: ParamSet<(
        Query<&mut Text, With<GameplayHudScore>>,
        Query<&mut Text, With<GameplayHudAccuracy>>,
        Query<&mut Text, With<GameplayHudCombo>>,
        Query<&mut Text, With<GameplayHudCounters>>,
        Query<&mut Text, With<GameplayHudGauge>>,
        Query<&mut Text, With<GameplayHudDebugText>>,
    )>,
    mut gauge_fill: Query<(&mut Node, &mut BackgroundColor), With<GameplayHudGaugeFill>>,
) {
    let score_text = format!("Score {:07}", display_score(&run));
    if let Ok(mut text) = hud_text.p0().single_mut() {
        set_text_if_changed(&mut text, &mut cache.score_text, score_text);
    }

    let accuracy_text = format!("Acc {:.2}%", accuracy_pct(&run));
    if let Ok(mut text) = hud_text.p1().single_mut() {
        set_text_if_changed(&mut text, &mut cache.accuracy_text, accuracy_text);
    }

    let combo_text = format!("Combo {} / Max {}", run.combo, run.max_combo);
    if let Ok(mut text) = hud_text.p2().single_mut() {
        set_text_if_changed(&mut text, &mut cache.combo_text, combo_text);
    }

    let counters_text = format!(
        "P:{} G:{} Good:{} Poor:{} Miss:{}",
        run.perfect, run.great, run.good, run.poor, run.miss,
    );
    if let Ok(mut text) = hud_text.p3().single_mut() {
        set_text_if_changed(&mut text, &mut cache.counters_text, counters_text);
    }

    let gauge_text = format!("Gauge {:.0}%", run.gauge * 100.0);
    if let Ok(mut text) = hud_text.p4().single_mut() {
        set_text_if_changed(&mut text, &mut cache.gauge_text, gauge_text);
    }

    let gauge_pct = run.gauge.clamp(0.0, 1.0);
    if cache.gauge_fill_pct != Some(gauge_pct) || cache.gauge_fill_failed != Some(run.failed) {
        if let Ok((mut node, mut color)) = gauge_fill.single_mut() {
            node.width = percent(gauge_pct * 100.0);
            color.0 = gauge_fill_color(run.gauge, run.failed);
        }
        cache.gauge_fill_pct = Some(gauge_pct);
        cache.gauge_fill_failed = Some(run.failed);
    }

    if !run.show_debug_hud {
        return;
    }

    let time_text = if run.elapsed < 0.0 {
        format!("starts in {:.1}", -run.elapsed)
    } else if run.failed {
        "FAILED".to_string()
    } else if run.finished || chart_notes_complete(&chart.notes) {
        "FINISHED".to_string()
    } else {
        format!("{:.1}s", run.elapsed)
    };
    let px_per_sec = layout.scroll_px_per_sec(run.lane_speed);
    let present_mode_text = if let Ok(window) = windows.single() {
        format!("{:?}", window.present_mode)
    } else {
        "n/a".to_string()
    };

    let live_voices = count_tracked_live_voices(&active, &audio_instances);
    let pending_decode = decode.as_ref().map(|r| r.pending).unwrap_or(0);

    let debug_text = format!(
        "{} | {} | {} | {}\nTime {}  Audio {:.3}s  Visual {:.3}s  Offset {:+.0}ms  {}{}\nScroll {:.2}x ({:.0}px/s)  Song {:.2}x  Vol M/B/D {:.0}/{:.0}/{:.0}%  Metro {}  LPmute {}  HitSound {}\nRender {:.1}fps {:.2}ms  Cap {} ({})  Drift a/v/c {:+.1}/{:+.1}/{:+.1}ms\nDiag spikes={} peak={:.1}ms metro={} preempt={} despawn={} decode={} midi={} voices={live_voices} decode_pending={pending_decode}",
        chart.title,
        chart.source,
        run.play_mode.label(),
        "BD SD FT HH LT HT CY RD LP LC",
        time_text,
        clock.audio_elapsed,
        clock.visual_elapsed,
        run.timing_offset * 1000.0,
        if is_paused(pause_state.get()) {
            "PAUSED "
        } else {
            ""
        },
        if run.failed {
            "FAILED"
        } else if run.finished || chart_notes_complete(&chart.notes) {
            "FINISHED"
        } else {
            ""
        },
        run.lane_speed,
        px_per_sec,
        run.song_playback_rate,
        mix.master * 100.0,
        mix.bgm * 100.0,
        mix.drums * 100.0,
        if run.metronome_sound { "on" } else { "off" },
        if run.lp_muting { "on" } else { "off" },
        if run.drum_hit_sound { "on" } else { "off" },
        stats.fps,
        stats.frame_ms,
        present_mode_text,
        config.fps_cap.label(),
        clock.audio_step_ms,
        clock.visual_drift_ms,
        clock.visual_correction_ms,
        playback_diag.frame_spikes,
        playback_diag.peak_frame_ms,
        playback_diag.metronome_beats,
        playback_diag.voice_preempts,
        playback_diag.metro_line_despawns,
        playback_diag.bg_decode_merges,
        playback_diag.midi_rescans,
    );
    if let Ok(mut text) = hud_text.p5().single_mut() {
        set_text_if_changed(&mut text, &mut cache.debug_text, debug_text);
    }
}

pub(crate) fn update_judgement_text(
    chart: Res<Chart>,
    run: Res<RunState>,
    mut cache: ResMut<HudDisplayCache>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<JudgementText>>,
) {
    if let Ok((mut text, mut color)) = text_query.single_mut() {
        let (new_text, new_color) = if run.failed {
            (
                format!("FAILED\nGauge {:.0}%", run.gauge * 100.0),
                Color::srgb(1.0, 0.25, 0.3),
            )
        } else if run.finished || chart_notes_complete(&chart.notes) {
            (
                format!("FINISH\nMAX COMBO {}", run.max_combo),
                Color::srgb(0.45, 0.95, 1.0),
            )
        } else if run.judgement_timer.is_finished() {
            if run.started {
                (String::new(), color.0)
            } else {
                ("READY".to_string(), color.0)
            }
        } else {
            (
                format!("{}\n{:+.0} ms", run.last_message, run.last_delta_ms),
                run.last_judgement.color(),
            )
        };

        if cache.judgement_text.as_ref() != Some(&new_text) {
            text.0 = new_text.clone();
            cache.judgement_text = Some(new_text);
        }
        if cache.judgement_color != Some(new_color) {
            color.0 = new_color;
            cache.judgement_color = Some(new_color);
        }
    }
}
