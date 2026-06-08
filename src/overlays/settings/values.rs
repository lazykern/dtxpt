use bevy::prelude::*;
use bevy::window::{PresentMode, Window};
use bevy::winit::WinitSettings;

use dtxpt::input::{InputBindings, MidiInputState, PlayMode, SystemAction};

use crate::app::state::{AppState, OverlayState, PauseState, is_paused};
use crate::audio::AudioMix;
use crate::config::{GameConfig, HitSoundPriority};
use crate::gameplay::constants::*;
use crate::gameplay::hotkeys::winit_settings_for_vsync;
use crate::gameplay::run::RunState;

use super::rows::SettingRow;

pub(crate) fn apply_vsync_setting(
    window: &mut Window,
    winit: Option<&mut WinitSettings>,
    enabled: bool,
) {
    window.present_mode = if enabled {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
    if let Some(winit) = winit {
        *winit = winit_settings_for_vsync(enabled);
    }
}

pub(crate) fn apply_setting_delta(
    row: SettingRow,
    delta: f32,
    config: &mut GameConfig,
    mix: &mut AudioMix,
    run: Option<&mut RunState>,
    window: Option<&mut Window>,
    winit: Option<&mut WinitSettings>,
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
                    PlayMode::Normal => PlayMode::Auto,
                    PlayMode::Auto => PlayMode::Practice,
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
                apply_vsync_setting(window, winit, config.vsync);
            } else if let Some(winit) = winit {
                *winit = winit_settings_for_vsync(config.vsync);
            }
        }
        SettingRow::MetronomeSound => {
            config.metronome_sound = !config.metronome_sound;
            if let Some(run) = run {
                run.metronome_sound = config.metronome_sound;
            }
        }
        SettingRow::LpMuting => {
            config.lp_muting = !config.lp_muting;
            if let Some(run) = run {
                run.lp_muting = config.lp_muting;
            }
        }
        SettingRow::DrumHitSound => {
            config.drum_hit_sound = !config.drum_hit_sound;
            if let Some(run) = run {
                run.drum_hit_sound = config.drum_hit_sound;
            }
        }
        SettingRow::HitSoundPriorityHh => {
            config.hit_sound_priority_hh =
                cycle_hit_sound_priority(config.hit_sound_priority_hh, delta);
            if let Some(run) = run {
                run.hit_sound_priority_hh = config.hit_sound_priority_hh;
            }
        }
        SettingRow::HitSoundPriorityTom => {
            config.hit_sound_priority_ft =
                cycle_hit_sound_priority(config.hit_sound_priority_ft, delta);
            if let Some(run) = run {
                run.hit_sound_priority_ft = config.hit_sound_priority_ft;
            }
        }
        SettingRow::HitSoundPriorityCymbal => {
            config.hit_sound_priority_cy =
                cycle_hit_sound_priority(config.hit_sound_priority_cy, delta);
            if let Some(run) = run {
                run.hit_sound_priority_cy = config.hit_sound_priority_cy;
            }
        }
        SettingRow::HitSoundPriorityBd => {
            config.hit_sound_priority_lp =
                cycle_hit_sound_priority(config.hit_sound_priority_lp, delta);
            if let Some(run) = run {
                run.hit_sound_priority_lp = config.hit_sound_priority_lp;
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

fn cycle_hit_sound_priority(current: HitSoundPriority, delta: f32) -> HitSoundPriority {
    if delta > 0.0 {
        current.next()
    } else {
        match current {
            HitSoundPriority::ChipOverPad => HitSoundPriority::PadOverChip,
            HitSoundPriority::PadOverChip => HitSoundPriority::ChipOverPad,
        }
    }
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
