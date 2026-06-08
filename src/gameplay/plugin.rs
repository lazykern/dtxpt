use bevy::prelude::*;

use crate::app::state::{AppState, PauseState, overlay_closed};
use crate::audio::{
    adjust_audio_mix, adjust_song_playback_rate, advance_audio_frame, check_song_finished,
    cleanup_active_sounds, merge_decoded_audio, playback_transport, restart_on_gesture,
    schedule_auto_se, schedule_metronome, sync_elapsed_from_audio,
};
use crate::gameplay::diagnostics::{
    PlaybackDiagnostics, monitor_playback_diagnostics, reset_playback_diagnostics,
};
use crate::gameplay::gauge::update_gauge_bar;
use crate::gameplay::hotkeys::toggle_hotkeys;
use crate::gameplay::hud::{
    HudDisplayCache, sync_debug_hud_visibility, update_hud, update_judgement_text,
    update_render_stats,
};
use crate::gameplay::input::{
    PendingLaneInputs, capture_lane_inputs, process_pending_lane_hits,
};
use crate::gameplay::judgement::miss_late_notes;
use crate::gameplay::layout::{
    apply_key_cap_layout, apply_playfield_layout, sync_playfield_layout,
};
use crate::gameplay::live_tuning::{adjust_lane_speed, adjust_timing_offset};
use crate::gameplay::rendering::playfield_viz::{
    update_hit_bursts, update_lane_receptor_flashes, update_metronome_lines, update_note_visuals,
};
use crate::gameplay::scoring::finish_to_result;
use crate::gameplay::setup::{cleanup_gameplay, setup_gameplay};
use crate::overlays::pause::{
    PauseUiState, pause_input, sync_pause_focus, toggle_playback_pause, update_pause_overlay,
};
use crate::overlays::settings::persist_runtime_config;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlaybackDiagnostics>()
            .init_resource::<crate::audio::RestartGestureState>()
            .init_resource::<PauseUiState>()
            .init_resource::<HudDisplayCache>()
            .init_resource::<PendingLaneInputs>()
            .add_systems(OnEnter(AppState::Playing), setup_gameplay)
            .add_systems(OnEnter(AppState::Playing), reset_playback_diagnostics)
            .add_systems(
                OnEnter(AppState::Playing),
                |mut next: ResMut<NextState<PauseState>>| next.set(PauseState::Running),
            )
            .add_systems(OnExit(AppState::Playing), cleanup_gameplay)
            .add_systems(
                Update,
                (
                    advance_audio_frame,
                    sync_elapsed_from_audio.after(advance_audio_frame),
                    restart_on_gesture.after(sync_elapsed_from_audio),
                    adjust_timing_offset
                        .after(restart_on_gesture)
                        .run_if(overlay_closed),
                    adjust_lane_speed
                        .after(adjust_timing_offset)
                        .run_if(overlay_closed),
                    adjust_song_playback_rate
                        .after(adjust_lane_speed)
                        .run_if(overlay_closed),
                    adjust_audio_mix
                        .after(adjust_song_playback_rate)
                        .run_if(overlay_closed),
                    persist_runtime_config.after(adjust_audio_mix),
                    toggle_playback_pause
                        .after(persist_runtime_config)
                        .run_if(overlay_closed),
                    playback_transport
                        .after(toggle_playback_pause)
                        .run_if(overlay_closed),
                    update_pause_overlay.after(playback_transport),
                    pause_input
                        .after(update_pause_overlay)
                        .run_if(overlay_closed),
                    sync_pause_focus.after(pause_input),
                    toggle_hotkeys
                        .after(sync_pause_focus)
                        .run_if(overlay_closed),
                    miss_late_notes.after(process_pending_lane_hits),
                    cleanup_active_sounds.after(process_pending_lane_hits),
                    schedule_auto_se.after(process_pending_lane_hits),
                    schedule_metronome.after(sync_elapsed_from_audio),
                    merge_decoded_audio,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    capture_lane_inputs
                        .after(sync_elapsed_from_audio)
                        .after(toggle_playback_pause)
                        .run_if(overlay_closed),
                    process_pending_lane_hits
                        .after(capture_lane_inputs)
                        .after(toggle_hotkeys),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    check_song_finished.after(sync_elapsed_from_audio),
                    finish_to_result
                        .after(miss_late_notes)
                        .after(check_song_finished),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    update_note_visuals
                        .after(sync_elapsed_from_audio)
                        .after(apply_playfield_layout),
                    update_metronome_lines
                        .after(sync_elapsed_from_audio)
                        .after(apply_playfield_layout),
                    update_hit_bursts,
                    update_lane_receptor_flashes,
                    crate::gameplay::rendering::keyboard_viz::update_key_cap_flashes,
                    update_render_stats.after(sync_elapsed_from_audio),
                    monitor_playback_diagnostics.after(update_render_stats),
                    sync_debug_hud_visibility,
                    update_hud.after(monitor_playback_diagnostics),
                    update_gauge_bar.after(apply_playfield_layout),
                    update_judgement_text,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    sync_playfield_layout,
                    apply_playfield_layout.after(sync_playfield_layout),
                    apply_key_cap_layout.after(apply_playfield_layout),
                ),
            );
    }
}
