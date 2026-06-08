use std::time::Duration;

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
use crate::gameplay::interp::{RenderVisualClock, interp_visual_clock};
use crate::gameplay::judgement::autoplay_hit_notes;
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
        // FixedUpdate cadence. 60Hz matches monitor refresh at 60fps and the
        // chart/SE timing assumptions. Default Bevy is 64Hz (a power of 2 for
        // lossless f32 conversion); we override to 60Hz for rhythm-game
        // consistency.
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f64(1.0 / 60.0)));
        app.init_resource::<PlaybackDiagnostics>()
            .init_resource::<RenderVisualClock>()
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
                RunFixedMainLoop,
                interp_visual_clock.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            )
            .add_systems(
                PreUpdate,
                capture_lane_inputs
                    .after(bevy::input::InputSystems)
                    .run_if(in_state(AppState::Playing).and(overlay_closed)),
            )
            .add_systems(
                FixedUpdate,
                (
                    sync_elapsed_from_audio,
                    process_pending_lane_hits,
                    autoplay_hit_notes,
                    miss_late_notes,
                    schedule_auto_se,
                    schedule_metronome,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    advance_audio_frame,
                    restart_on_gesture,
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
                    cleanup_active_sounds,
                    merge_decoded_audio,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    check_song_finished,
                    finish_to_result.after(check_song_finished),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    update_note_visuals.after(apply_playfield_layout),
                    update_metronome_lines.after(apply_playfield_layout),
                    update_hit_bursts,
                    update_lane_receptor_flashes,
                    crate::gameplay::rendering::keyboard_viz::update_key_cap_flashes,
                    update_render_stats,
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
