use bevy::prelude::*;
use bevy::window::Window;
use bevy::winit::WinitSettings;
use bevy_framepace::FramepaceSettings;

use dtxpt::chart::Judgement;
use dtxpt::input::{InputBindings, MidiInputState, SystemAction};

use crate::app::state::{AppState, OverlayState, PauseState, is_paused};
use crate::audio::AudioMix;
use crate::config::{
    BDGroup, CYGroup, DamageLevel, DarkMode, FTGroup, FpsCap, GameConfig, GaugeMode, HHGroup,
    HitSoundPriority, RDPosition, RandomMode,
};
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
    row: &SettingRow,
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
            if let Some(run) = run
                && run.practice != config.practice_song_select
            {
                run.practice = config.practice_song_select;
                reset_run_for_mode_change(run);
            }
        }
        SettingRow::SkillMode => {
            config.skill_mode = config.skill_mode.next();
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
            if config.per_lane_auto.contains(lane) {
                config.per_lane_auto.remove(lane);
            } else {
                config.per_lane_auto.insert(*lane);
            }
            // In Normal mode the run is locked; per_lane changes only take
            // effect at next run start. In Practice we propagate live.
            if let Some(run) = run
                && run.practice
            {
                run.active_mods.auto_lanes =
                    resolve_auto_lanes(&config.per_lane_auto, config.auto_mode);
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
        SettingRow::PedalLagTime => {
            config.pedal_lag_time_ms =
                (config.pedal_lag_time_ms + delta.signum() as i32).clamp(-100, 100);
            if let Some(run) = run {
                run.pedal_lag_time_ms = config.pedal_lag_time_ms;
            }
        }
        SettingRow::TimingOffset => {
            config.timing_offset =
                (config.timing_offset + delta * TIMING_OFFSET_STEP).clamp(-0.5, 0.5);
            if let Some(run) = run {
                run.timing_offset = config.timing_offset;
            }
        }
        SettingRow::GuitarTimingOffset => {
            config.guitar_offset =
                (config.guitar_offset + delta * TIMING_OFFSET_STEP).clamp(-0.5, 0.5);
            if let Some(run) = run {
                run.guitar_offset = config.guitar_offset;
            }
        }
        SettingRow::BassTimingOffset => {
            config.bass_offset = (config.bass_offset + delta * TIMING_OFFSET_STEP).clamp(-0.5, 0.5);
            if let Some(run) = run {
                run.bass_offset = config.bass_offset;
            }
        }
        SettingRow::PlaySpeedNum => {
            config.play_speed_num =
                (config.play_speed_num as i32 + delta.signum() as i32).clamp(1, 100) as u32;
        }
        SettingRow::PlaySpeedDen => {
            config.play_speed_den =
                (config.play_speed_den as i32 + delta.signum() as i32).clamp(1, 100) as u32;
        }
        SettingRow::SaveScoreIfModifiedPlaySpeed => {
            config.save_score_if_modified_play_speed = !config.save_score_if_modified_play_speed;
        }
        SettingRow::HhGroup => {
            config.hh_group = cycle_enum(config.hh_group, delta, |g| match g {
                HHGroup::AllSplit => HHGroup::HhOnlySplit,
                HHGroup::HhOnlySplit => HHGroup::LcOnlySplit,
                HHGroup::LcOnlySplit => HHGroup::AllCommon,
                HHGroup::AllCommon => HHGroup::AllSplit,
            });
        }
        SettingRow::FtGroup => {
            config.ft_group = cycle_enum(config.ft_group, delta, |g| match g {
                FTGroup::Split => FTGroup::Common,
                FTGroup::Common => FTGroup::Split,
            });
        }
        SettingRow::CyGroup => {
            config.cy_group = cycle_enum(config.cy_group, delta, |g| match g {
                CYGroup::Split => CYGroup::Common,
                CYGroup::Common => CYGroup::Split,
            });
        }
        SettingRow::BdGroup => {
            config.bd_group = cycle_enum(config.bd_group, delta, |g| match g {
                BDGroup::Split => BDGroup::BdAndLp,
                BDGroup::BdAndLp => BDGroup::LpPair,
                BDGroup::LpPair => BDGroup::BothBd,
                BDGroup::BothBd => BDGroup::Split,
            });
        }
        SettingRow::RdPosition => {
            config.rd_position = cycle_enum(config.rd_position, delta, |p| match p {
                RDPosition::RdRc => RDPosition::RcRd,
                RDPosition::RcRd => RDPosition::RdRc,
            });
        }
        SettingRow::DarkMode => {
            config.dark = cycle_enum(config.dark, delta, |d| match d {
                DarkMode::Off => DarkMode::Half,
                DarkMode::Half => DarkMode::Full,
                DarkMode::Full => DarkMode::Off,
            });
        }
        SettingRow::RandomMode => {
            config.random = cycle_enum(config.random, delta, |r| match r {
                RandomMode::Off => RandomMode::Mirror,
                RandomMode::Mirror => RandomMode::Random,
                RandomMode::Random => RandomMode::SuperRandom,
                RandomMode::SuperRandom => RandomMode::HyperRandom,
                RandomMode::HyperRandom => RandomMode::MasterRandom,
                RandomMode::MasterRandom => RandomMode::AnotherRandom,
                RandomMode::AnotherRandom => RandomMode::Off,
            });
        }
        SettingRow::GaugeMode => {
            config.gauge.mode = cycle_enum(config.gauge.mode, delta, |m| match m {
                GaugeMode::Normal => GaugeMode::Hard,
                GaugeMode::Hard => GaugeMode::Death,
                GaugeMode::Death => GaugeMode::Extreme,
                GaugeMode::Extreme => GaugeMode::ExHard,
                GaugeMode::ExHard => GaugeMode::Normal,
            });
        }
        SettingRow::Risky => {
            // Risky: 0 = off, 1..=10 = Risky N. Negative deltas cycle
            // back through 10.
            let current = config.gauge.risky_initial;
            let next = if delta > 0.0 {
                (current + 1).min(10)
            } else {
                current.checked_sub(1).unwrap_or(10)
            };
            config.gauge.risky_initial = next;
            if let Some(run) = run {
                run.risky_initial = next;
                run.risky_times_remaining = next;
            }
        }
        SettingRow::DamageLevel => {
            config.gauge.damage_level = cycle_enum(config.gauge.damage_level, delta, |d| match d {
                DamageLevel::Small => DamageLevel::Normal,
                DamageLevel::Normal => DamageLevel::High,
                DamageLevel::High => DamageLevel::Small,
            });
            if let Some(run) = run {
                run.damage_level = config.gauge.damage_level;
            }
        }
        SettingRow::AutoAddGage => {
            config.gauge.auto_add_gauge = !config.gauge.auto_add_gauge;
            if let Some(run) = run {
                run.auto_add_gauge = config.gauge.auto_add_gauge;
            }
        }
        SettingRow::StoicMode => {
            config.stoic_mode = !config.stoic_mode;
        }
        SettingRow::CompactMode => {
            config.compact_mode = !config.compact_mode;
        }
        SettingRow::RandomSubBox => {
            config.random_sub_box = !config.random_sub_box;
        }
        SettingRow::WaveDriftCorrection => {
            config.wave_drift_correction = !config.wave_drift_correction;
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
        SettingRow::UseOsTimer => {
            config.use_os_timer = !config.use_os_timer;
        }
        SettingRow::ChipPlayTimeComputeMode => {
            config.chip_play_time_compute_mode = config.chip_play_time_compute_mode.next();
        }
        SettingRow::WriteScoreIni => {
            config.write_score_ini = !config.write_score_ini;
        }
        SettingRow::CymbalFree => {
            config.cymbal_free = !config.cymbal_free;
            if let Some(run) = run {
                run.cymbal_free = config.cymbal_free;
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

/// Cycle a small enum to its next (or previous) value. `next` maps
/// each variant to its successor. For positive deltas we apply
/// `next` once. For negative deltas we step forward through the
/// cycle until we find the value whose successor is `current`
/// (the "previous" value). Works for any cycle length (2..=7).
fn cycle_enum<T: Copy + PartialEq>(current: T, delta: f32, next: fn(T) -> T) -> T {
    if delta > 0.0 {
        return next(current);
    }
    let mut previous = current;
    let mut candidate = next(current);
    while candidate != current {
        previous = candidate;
        candidate = next(candidate);
    }
    previous
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_delta_cycles_to_previous_enum_variant() {
        let mut config = GameConfig::default();
        let mut mix = AudioMix::from_config(&config);

        assert_eq!(config.random, RandomMode::Off);
        assert!(apply_setting_delta(
            &SettingRow::RandomMode,
            -1.0,
            &mut config,
            &mut mix,
            None,
            None,
            None,
        ));
        assert_eq!(config.random, RandomMode::AnotherRandom);

        assert!(apply_setting_delta(
            &SettingRow::GaugeMode,
            -1.0,
            &mut config,
            &mut mix,
            None,
            None,
            None,
        ));
        assert_eq!(config.gauge.mode, GaugeMode::ExHard);
    }

    #[test]
    fn per_instrument_offsets_apply_live_to_config() {
        let mut config = GameConfig::default();
        let mut mix = AudioMix::from_config(&config);

        assert!(apply_setting_delta(
            &SettingRow::GuitarTimingOffset,
            1.0,
            &mut config,
            &mut mix,
            None,
            None,
            None,
        ));
        assert!(apply_setting_delta(
            &SettingRow::BassTimingOffset,
            -1.0,
            &mut config,
            &mut mix,
            None,
            None,
            None,
        ));

        assert!((config.guitar_offset - TIMING_OFFSET_STEP).abs() < f32::EPSILON);
        assert!((config.bass_offset + TIMING_OFFSET_STEP).abs() < f32::EPSILON);
    }
}
