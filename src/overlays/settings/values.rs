use bevy::prelude::*;
use bevy::window::Window;
use bevy::winit::WinitSettings;
use bevy_framepace::FramepaceSettings;

use dtxpt::chart::Judgement;
use dtxpt::input::{InputBindings, MidiInputState, SystemAction};

use crate::app::state::{AppState, OverlayState, PauseState, is_paused};
use crate::audio::AudioMix;
use crate::config::{FpsCap, GameConfig, HitSoundPriority};
use crate::gameplay::constants::*;
use crate::gameplay::gauge::GAUGE_START;
use crate::gameplay::mods::resolve_auto_lanes;
use crate::gameplay::run::RunState;

use super::rows::SettingRow;

pub(crate) fn apply_fps_cap(
    window: Option<&mut Window>,
    winit: Option<&mut WinitSettings>,
    framepace: Option<&mut FramepaceSettings>,
    cap: FpsCap,
) {
    if let Some(window) = window {
        window.present_mode = cap.present_mode();
    }
    if let Some(winit) = winit {
        *winit = cap.winit_settings();
    }
    if let Some(framepace) = framepace {
        framepace.limiter = cap.limiter();
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
    if delta == 0.0 || !row.live_adjustable(run.as_deref().map(|run| run.practice)) {
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
        SettingRow::Practice => {
            let prev = config.practice_song_select;
            config.practice_song_select = !prev;
            if let Some(run) = run {
                if run.practice != config.practice_song_select {
                    run.practice = config.practice_song_select;
                    reset_run_for_mode_change(run);
                }
            }
        }
        SettingRow::AutoMode => {
            let prev = config.auto_mode;
            config.auto_mode = prev.next();
            if let Some(run) = run {
                // Re-resolve effective auto set with the new mode.
                run.active_mods.auto_lanes =
                    resolve_auto_lanes(&config.per_lane_auto, config.auto_mode);
            }
        }
        SettingRow::PerLaneAuto(lane) => {
            if config.per_lane_auto.contains(&lane) {
                config.per_lane_auto.remove(&lane);
            } else {
                config.per_lane_auto.insert(lane);
            }
            // In Normal mode the run is locked; per_lane changes only take
            // effect at next run start. In Practice we propagate live.
            if let Some(run) = run {
                if run.practice {
                    run.active_mods.auto_lanes =
                        resolve_auto_lanes(&config.per_lane_auto, config.auto_mode);
                }
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
        SettingRow::FpsCap => {
            config.fps_cap = config.fps_cap.next();
            apply_fps_cap(window, winit, None, config.fps_cap);
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

/// Reset run counters/gauge so a freshly-chosen mode takes effect from
/// the current chart position. Keeps timing/position state (elapsed,
/// started, raw_elapsed) so the player can keep playing without seeking.
fn reset_run_for_mode_change(run: &mut RunState) {
    run.score = 0.0;
    run.judge_units = 0.0;
    run.combo = 0;
    run.max_combo = 0;
    run.perfect = 0;
    run.great = 0;
    run.good = 0;
    run.poor = 0;
    run.miss = 0;
    run.last_judgement = Judgement::Miss;
    run.last_message = "READY".into();
    run.last_delta_ms = 0.0;
    run.last_was_auto = false;
    run.judgement_timer = Timer::from_seconds(0.0, TimerMode::Once);
    run.finished = false;
    run.failed = false;
    run.gauge = GAUGE_START;
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
