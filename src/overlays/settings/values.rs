use bevy::prelude::*;
use bevy::window::{PresentMode, Window};

use dtxpt::input::{InputBindings, MidiInputState, PlayMode, SystemAction};

use crate::app::state::{AppState, OverlayState, PauseState, is_paused};
use crate::audio::AudioMix;
use crate::config::GameConfig;
use crate::gameplay::constants::*;
use crate::gameplay::run::RunState;

use super::rows::SettingRow;

pub(crate) fn apply_vsync_setting(window: &mut Window, enabled: bool) {
    window.present_mode = if enabled {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
}

pub(crate) fn apply_setting_delta(
    row: SettingRow,
    delta: f32,
    config: &mut GameConfig,
    mix: &mut AudioMix,
    run: Option<&mut RunState>,
    window: Option<&mut Window>,
) -> bool {
    if delta == 0.0 || !row.live_adjustable(run.as_deref().map(|run| run.play_mode)) {
        return false;
    }

    let before = config.clone();
    match row {
        SettingRow::ChartRoot => {}
        SettingRow::MasterVolume => {
            config.master_volume =
                (config.master_volume + f64::from(delta * VOLUME_STEP)).clamp(0.0, 1.0);
            mix.master = config.master_volume as f32;
        }
        SettingRow::BgmVolume => {
            config.bgm_volume =
                (config.bgm_volume + f64::from(delta * VOLUME_STEP)).clamp(0.0, 1.0);
            mix.bgm = config.bgm_volume as f32;
        }
        SettingRow::DrumVolume => {
            config.drum_volume =
                (config.drum_volume + f64::from(delta * VOLUME_STEP)).clamp(0.0, 1.0);
            mix.drums = config.drum_volume as f32;
        }
        SettingRow::PlayMode => {
            config.play_mode = if delta > 0.0 {
                config.play_mode.next()
            } else {
                match config.play_mode {
                    PlayMode::Normal => PlayMode::Practice,
                    PlayMode::Practice => PlayMode::Normal,
                }
            };
            if let Some(run) = run {
                run.play_mode = config.play_mode;
            }
        }
        SettingRow::LaneKey(_) | SettingRow::SystemAction(_) => {}
        SettingRow::LaneSpeed => {
            config.lane_speed =
                (config.lane_speed + delta * LANE_SPEED_STEP).clamp(MIN_LANE_SPEED, MAX_LANE_SPEED);
            if let Some(run) = run {
                run.lane_speed = config.lane_speed;
            }
        }
        SettingRow::TimingOffset => {
            config.timing_offset =
                (config.timing_offset + delta * TIMING_OFFSET_STEP).clamp(-0.5, 0.5);
            if let Some(run) = run {
                run.timing_offset = config.timing_offset;
            }
        }
        SettingRow::SongRate => {
            config.song_playback_rate = (config.song_playback_rate + delta * SONG_RATE_STEP)
                .clamp(MIN_SONG_PLAYBACK_RATE, MAX_SONG_PLAYBACK_RATE);
            if let Some(run) = run {
                run.song_playback_rate = config.song_playback_rate;
            }
        }
        SettingRow::Vsync => {
            config.vsync = !config.vsync;
            if let Some(window) = window {
                apply_vsync_setting(window, config.vsync);
            }
        }
        SettingRow::MetronomeSound => {
            config.metronome_sound = !config.metronome_sound;
            if let Some(run) = run {
                run.metronome_sound = config.metronome_sound;
            }
        }
        SettingRow::DebugHud => {
            config.show_debug_hud = !config.show_debug_hud;
            if let Some(run) = run {
                run.show_debug_hud = config.show_debug_hud;
            }
        }
    }
    before != *config
}

pub fn settings_overlay_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    bindings: Res<InputBindings>,
    app_state: Res<State<AppState>>,
    pause_state: Res<State<PauseState>>,
    overlay_state: Res<State<OverlayState>>,
    mut next_overlay: ResMut<NextState<OverlayState>>,
) {
    if !bindings.action_just_pressed(
        SystemAction::ToggleSettings,
        &keyboard,
        &midi.note_on_events,
    ) {
        return;
    }

    if *overlay_state.get() == OverlayState::Settings {
        next_overlay.set(OverlayState::None);
        return;
    }

    let can_open = match app_state.get() {
        AppState::MainMenu | AppState::SongSelect | AppState::Result => true,
        AppState::Playing => is_paused(pause_state.get()),
        _ => false,
    };
    if can_open {
        next_overlay.set(OverlayState::Settings);
    }
}
