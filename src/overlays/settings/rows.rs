use dtxpt::input::lanes::LANES;
use dtxpt::input::{PlayMode, SYSTEM_ACTION_SETTINGS_ORDER, SystemAction};

use crate::audio::AudioMix;
use crate::config::GameConfig;
use crate::gameplay::constants::*;
use crate::gameplay::live_tuning::{
    play_mode_change_allowed_during_play, song_rate_change_allowed_during_play,
};
use dtxpt::input::{keyboard_summary_for_lane, lane_bindings_value, system_action_binding_value};

use super::SettingsOverlay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SettingCategory {
    #[default]
    General,
    Audio,
    Gameplay,
    Input,
    Graphics,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingRow {
    ChartRoot,
    MasterVolume,
    BgmVolume,
    DrumVolume,
    TimingOffset,
    PlayMode,
    LaneSpeed,
    SongRate,
    LaneKey(usize),
    SystemAction(SystemAction),
    FpsCap,
    MetronomeSound,
    LpMuting,
    DrumHitSound,
    HitSoundPriorityHh,
    HitSoundPriorityTom,
    HitSoundPriorityCymbal,
    HitSoundPriorityBd,
    DebugHud,
}

pub(crate) fn category_rows(category: SettingCategory) -> Vec<SettingRow> {
    match category {
        SettingCategory::General => vec![SettingRow::ChartRoot],
        SettingCategory::Audio => vec![
            SettingRow::MasterVolume,
            SettingRow::BgmVolume,
            SettingRow::DrumVolume,
            SettingRow::TimingOffset,
        ],
        SettingCategory::Gameplay => vec![
            SettingRow::PlayMode,
            SettingRow::LaneSpeed,
            SettingRow::SongRate,
            SettingRow::LpMuting,
            SettingRow::DrumHitSound,
            SettingRow::HitSoundPriorityHh,
            SettingRow::HitSoundPriorityTom,
            SettingRow::HitSoundPriorityCymbal,
            SettingRow::HitSoundPriorityBd,
        ],
        SettingCategory::Input => {
            let mut rows: Vec<SettingRow> = (0..LANES.len()).map(SettingRow::LaneKey).collect();
            rows.extend(
                SYSTEM_ACTION_SETTINGS_ORDER
                    .iter()
                    .copied()
                    .map(SettingRow::SystemAction),
            );
            rows
        }
        SettingCategory::Graphics => vec![SettingRow::FpsCap],
        SettingCategory::Debug => vec![SettingRow::MetronomeSound, SettingRow::DebugHud],
    }
}

pub(crate) fn all_setting_rows() -> Vec<SettingRow> {
    [
        SettingCategory::General,
        SettingCategory::Audio,
        SettingCategory::Gameplay,
        SettingCategory::Input,
        SettingCategory::Graphics,
        SettingCategory::Debug,
    ]
    .into_iter()
    .flat_map(category_rows)
    .collect()
}

pub(crate) fn filtered_settings(search: &str, category: SettingCategory) -> Vec<SettingRow> {
    let search = search.trim().to_ascii_lowercase();
    let source = if search.is_empty() {
        category_rows(category)
    } else {
        all_setting_rows()
    };
    source
        .into_iter()
        .filter(|row| {
            if search.is_empty() {
                true
            } else {
                row.label().to_ascii_lowercase().contains(&search)
                    || row.description().to_ascii_lowercase().contains(&search)
                    || row
                        .category()
                        .label()
                        .to_ascii_lowercase()
                        .contains(&search)
            }
        })
        .collect()
}

impl SettingCategory {
    pub(crate) fn label(self) -> &'static str {
        match self {
            SettingCategory::General => "General",
            SettingCategory::Audio => "Audio",
            SettingCategory::Gameplay => "Gameplay",
            SettingCategory::Input => "Input",
            SettingCategory::Graphics => "Graphics",
            SettingCategory::Debug => "Debug",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            SettingCategory::General => SettingCategory::Audio,
            SettingCategory::Audio => SettingCategory::Gameplay,
            SettingCategory::Gameplay => SettingCategory::Input,
            SettingCategory::Input => SettingCategory::Graphics,
            SettingCategory::Graphics => SettingCategory::Debug,
            SettingCategory::Debug => SettingCategory::General,
        }
    }
}

impl SettingRow {
    pub(crate) fn category(self) -> SettingCategory {
        match self {
            SettingRow::ChartRoot => SettingCategory::General,
            SettingRow::MasterVolume
            | SettingRow::BgmVolume
            | SettingRow::DrumVolume
            | SettingRow::TimingOffset => SettingCategory::Audio,
            SettingRow::PlayMode | SettingRow::LaneSpeed | SettingRow::SongRate => {
                SettingCategory::Gameplay
            }
            SettingRow::LaneKey(_) | SettingRow::SystemAction(_) => SettingCategory::Input,
            SettingRow::FpsCap => SettingCategory::Graphics,
            SettingRow::MetronomeSound | SettingRow::DebugHud => SettingCategory::Debug,
            SettingRow::LpMuting | SettingRow::DrumHitSound => SettingCategory::Gameplay,
            SettingRow::HitSoundPriorityHh
            | SettingRow::HitSoundPriorityTom
            | SettingRow::HitSoundPriorityCymbal
            | SettingRow::HitSoundPriorityBd => SettingCategory::Gameplay,
        }
    }

    pub(crate) fn is_adjustable(self) -> bool {
        !matches!(
            self,
            SettingRow::ChartRoot | SettingRow::LaneKey(_) | SettingRow::SystemAction(_)
        )
    }

    pub(crate) fn live_adjustable(self, active_play_mode: Option<PlayMode>) -> bool {
        if !self.is_adjustable() {
            return false;
        }
        match self {
            SettingRow::PlayMode => play_mode_change_allowed_during_play(active_play_mode),
            SettingRow::SongRate => song_rate_change_allowed_during_play(active_play_mode),
            _ => true,
        }
    }

    pub(crate) fn is_toggle(self) -> bool {
        matches!(
            self,
            SettingRow::MetronomeSound
                | SettingRow::LpMuting
                | SettingRow::DrumHitSound
                | SettingRow::DebugHud
        )
    }

    pub(crate) fn toggle_value(self, config: &GameConfig) -> bool {
        match self {
            SettingRow::MetronomeSound => config.metronome_sound,
            SettingRow::LpMuting => config.lp_muting,
            SettingRow::DrumHitSound => config.drum_hit_sound,
            SettingRow::DebugHud => config.show_debug_hud,
            _ => false,
        }
    }

    pub(crate) fn slider_ratio(self, config: &GameConfig, mix: &AudioMix) -> Option<f32> {
        match self {
            SettingRow::MasterVolume => Some(mix.master),
            SettingRow::BgmVolume => Some(mix.bgm),
            SettingRow::DrumVolume => Some(mix.drums),
            SettingRow::TimingOffset => Some(((config.timing_offset + 0.5) / 1.0).clamp(0.0, 1.0)),
            SettingRow::LaneSpeed => Some(
                ((config.lane_speed - MIN_LANE_SPEED) / (MAX_LANE_SPEED - MIN_LANE_SPEED))
                    .clamp(0.0, 1.0),
            ),
            SettingRow::SongRate => Some(
                ((config.song_playback_rate - MIN_SONG_PLAYBACK_RATE)
                    / (MAX_SONG_PLAYBACK_RATE - MIN_SONG_PLAYBACK_RATE))
                    .clamp(0.0, 1.0),
            ),
            _ => None,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            SettingRow::ChartRoot => "Chart root",
            SettingRow::MasterVolume => "Master volume",
            SettingRow::BgmVolume => "BGM volume",
            SettingRow::DrumVolume => "Drum volume",
            SettingRow::PlayMode => "Play mode",
            SettingRow::LaneSpeed => "Lane speed",
            SettingRow::LaneKey(lane) => LANES[lane].label,
            SettingRow::SystemAction(action) => action.label(),
            SettingRow::TimingOffset => "Timing offset",
            SettingRow::SongRate => "Song rate",
            SettingRow::FpsCap => "FPS cap",
            SettingRow::MetronomeSound => "Metronome sound",
            SettingRow::LpMuting => "LP muting",
            SettingRow::DrumHitSound => "Drum hit sound",
            SettingRow::HitSoundPriorityHh => "HH hit priority",
            SettingRow::HitSoundPriorityTom => "Tom hit priority",
            SettingRow::HitSoundPriorityCymbal => "Cymbal hit priority",
            SettingRow::HitSoundPriorityBd => "BD hit priority",
            SettingRow::DebugHud => "Debug HUD",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            SettingRow::ChartRoot => "Folder scanned for songs. Edit config.ron to change.",
            SettingRow::MasterVolume => "Global output level.",
            SettingRow::BgmVolume => "Song backing track level.",
            SettingRow::DrumVolume => "Drum hit and auto-SE level.",
            SettingRow::PlayMode => {
                "Normal or Practice. Locked mid-chart — choose before starting."
            }
            SettingRow::LaneSpeed => "Note scroll speed. Applies immediately during play.",
            SettingRow::LaneKey(_) => {
                "Lane bindings. Enter adds key or MIDI pad. ←/→ select binding. Backspace/Delete removes selected."
            }
            SettingRow::SystemAction(action) => action.description(),
            SettingRow::TimingOffset => "Visual/judgement offset in milliseconds.",
            SettingRow::SongRate => {
                "Song playback rate. Practice mode only during play; always editable from menus."
            }
            SettingRow::FpsCap => "Frame rate cap (VSync / 60 / 120 / 144 / 240 / Unlimited). Hotkey: F6.",
            SettingRow::MetronomeSound => "Play metronome clicks during gameplay.",
            SettingRow::LpMuting => "Let left pedal close/choke hi-hat WAVs like DTXMania.",
            SettingRow::DrumHitSound => "Play chart WAV when you manually hit drum notes.",
            SettingRow::HitSoundPriorityHh => {
                "HH group (HH/HHO/LC): chip = hit note WAV, pad = nearest group note WAV."
            }
            SettingRow::HitSoundPriorityTom => {
                "Tom group (LT/FT): chip = hit note WAV, pad = nearest group note WAV."
            }
            SettingRow::HitSoundPriorityCymbal => {
                "Cymbal group (CY/RD): chip = hit note WAV, pad = nearest group note WAV."
            }
            SettingRow::HitSoundPriorityBd => {
                "BD group (BD/LP): chip = hit note WAV, pad = nearest group note WAV."
            }
            SettingRow::DebugHud => "Show gameplay debug HUD overlay.",
        }
    }

    pub(crate) fn value(self, config: &GameConfig, mix: &AudioMix) -> String {
        match self {
            SettingRow::LaneKey(lane) => keyboard_summary_for_lane(&config.bindings, lane),
            _ => self.value_with_overlay(config, mix, &SettingsOverlay::default()),
        }
    }

    pub(crate) fn value_with_overlay(
        self,
        config: &GameConfig,
        mix: &AudioMix,
        overlay: &SettingsOverlay,
    ) -> String {
        match self {
            SettingRow::ChartRoot => config.chart_root.clone(),
            SettingRow::MasterVolume => format!("{:.0}%", mix.master * 100.0),
            SettingRow::BgmVolume => format!("{:.0}%", mix.bgm * 100.0),
            SettingRow::DrumVolume => format!("{:.0}%", mix.drums * 100.0),
            SettingRow::PlayMode => config.play_mode.label().to_string(),
            SettingRow::HitSoundPriorityHh => config.hit_sound_priority_hh.label().to_string(),
            SettingRow::HitSoundPriorityTom => config.hit_sound_priority_ft.label().to_string(),
            SettingRow::HitSoundPriorityCymbal => config.hit_sound_priority_cy.label().to_string(),
            SettingRow::HitSoundPriorityBd => config.hit_sound_priority_lp.label().to_string(),
            SettingRow::LaneSpeed => format!("{:.2}x", config.lane_speed),
            SettingRow::LaneKey(lane) => {
                lane_bindings_value(&config.bindings, lane, Some(overlay.lane_binding_cursor))
            }
            SettingRow::SystemAction(action) => {
                system_action_binding_value(&config.bindings, action)
            }
            SettingRow::TimingOffset => format!("{:+.0}ms", config.timing_offset * 1000.0),
            SettingRow::SongRate => format!("{:.2}x", config.song_playback_rate),
            SettingRow::FpsCap => config.fps_cap.label().to_string(),
            SettingRow::MetronomeSound => on_off(config.metronome_sound),
            SettingRow::LpMuting => on_off(config.lp_muting),
            SettingRow::DrumHitSound => on_off(config.drum_hit_sound),
            SettingRow::DebugHud => on_off(config.show_debug_hud),
        }
    }
}

fn on_off(value: bool) -> String {
    if value { "on".into() } else { "off".into() }
}
