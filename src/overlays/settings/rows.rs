use dtxpt::input::bindings::{BassLane, DrumLane, GuitarLane};
use dtxpt::input::lanes::LANES;
use dtxpt::input::{BindingTarget, SYSTEM_ACTION_SETTINGS_ORDER, SystemAction};

use crate::audio::AudioMix;
use crate::config::{
    BDGroup, CYGroup, DamageLevel, DarkMode, FTGroup, GameConfig, GaugeMode, HHGroup, RDPosition,
    RandomMode,
};
use crate::gameplay::constants::*;
use crate::gameplay::live_tuning::{
    play_mode_change_allowed_during_play, song_rate_change_allowed_during_play,
};
use dtxpt::input::{
    keyboard_summary_for_target, system_action_binding_value, target_bindings_value,
};

use super::SettingsOverlay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SettingCategory {
    #[default]
    General,
    Audio,
    Gameplay,
    Drums,
    Input,
    InputDrums,
    InputGuitar,
    InputBass,
    Graphics,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SettingRow {
    ChartRoot,
    MasterVolume,
    BgmVolume,
    DrumVolume,
    /// Drum timing offset in seconds. Equivalent to the legacy global
    /// `timing_offset`. See also `GuitarTimingOffset` / `BassTimingOffset`
    /// for per-instrument calibration (Phase D.3).
    TimingOffset,
    /// Per-instrument guitar timing offset.
    GuitarTimingOffset,
    /// Per-instrument bass timing offset.
    BassTimingOffset,
    Practice,
    SkillMode,
    AutoMode,
    PerLaneAuto(DrumLane),
    LaneSpeed,
    PedalLagTime,
    SongRate,
    /// Numerator for `PlaySpeed=N/D` (BocuD `nPlaySpeedNumerator`,
    /// default 20).
    PlaySpeedNum,
    /// Denominator for `PlaySpeed=N/D` (BocuD `nPlaySpeedDenominator`,
    /// default 20).
    PlaySpeedDen,
    /// BocuD `bSaveScoreIfModifiedPlaySpeed`. When OFF, runs at
    /// non-1.0× play speed are excluded from the best board.
    SaveScoreIfModifiedPlaySpeed,
    CymbalFree,
    /// One row per (instrument, lane) combination. The drum
    /// `usize` index is interpreted via [`LANES`](dtxpt::input::LANES);
    /// guitar/bass carry a fully-typed [`BindingTarget`].
    LaneKey(BindingTarget),
    SystemAction(SystemAction),
    FpsCap,
    MetronomeSound,
    UseOsTimer,
    ChipPlayTimeComputeMode,
    WriteScoreIni,
    LpMuting,
    DrumHitSound,
    HitSoundPriorityHh,
    HitSoundPriorityTom,
    HitSoundPriorityCymbal,
    HitSoundPriorityBd,
    /// BocuD `EHHGroup`. HH/LC pad grouping policy.
    HhGroup,
    /// BocuD `EFTGroup`. FT/HT/LT tom grouping policy.
    FtGroup,
    /// BocuD `ECYGroup`. CY/RD/LC cymbal grouping policy.
    CyGroup,
    /// BocuD `EBDGroup`. BD/LP/LBD pedal grouping policy.
    BdGroup,
    /// BocuD `ERDPosition`. Physical order of the RD and CY lanes.
    RdPosition,
    /// BocuD `EDarkMode`. Note visibility policy.
    DarkMode,
    /// BocuD `ERandomMode` (7 variants: Off/Mirror/Random/Super/
    /// Hyper/Master/Another). Applied to drum pads.
    RandomMode,
    /// BocuD gauge `GAUGE_*` difficulty factor index.
    GaugeMode,
    /// Risky mode initial miss count (0 = off, 1..=10 = Risky N).
    Risky,
    /// BocuD `EDamageLevel`. Multiplier on Poor/Miss gauge delta.
    DamageLevel,
    /// BocuD `bAutoAddGage`. Auto-played chips contribute positive
    /// gauge delta.
    AutoAddGage,
    /// BocuD `bストイックモード`. Suppresses in-play animations and
    /// song-select preimage animation.
    StoicMode,
    /// BocuD `bCompactMode`. Skip Title stage on startup.
    CompactMode,
    /// BocuD `bランダムセレクトで子BOXを検索対象とする`. Random song
    /// select descends into sub-boxes (Phase F wiring).
    RandomSubBox,
    /// BocuD `bWave再生位置自動調整機能有効`. Kira already handles
    /// long-WAV position correction; this is the user-facing toggle.
    WaveDriftCorrection,
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
        SettingCategory::Gameplay => {
            let mut rows: Vec<SettingRow> = vec![
                SettingRow::Practice,
                SettingRow::SkillMode,
                SettingRow::AutoMode,
            ];
            rows.extend(DrumLane::ALL.iter().copied().map(SettingRow::PerLaneAuto));
            rows.push(SettingRow::LaneSpeed);
            rows.push(SettingRow::PedalLagTime);
            rows.push(SettingRow::SongRate);
            rows.push(SettingRow::PlaySpeedNum);
            rows.push(SettingRow::PlaySpeedDen);
            rows.push(SettingRow::SaveScoreIfModifiedPlaySpeed);
            rows.push(SettingRow::CymbalFree);
            rows.push(SettingRow::LpMuting);
            rows.push(SettingRow::DrumHitSound);
            rows.push(SettingRow::HitSoundPriorityHh);
            rows.push(SettingRow::HitSoundPriorityTom);
            rows.push(SettingRow::HitSoundPriorityCymbal);
            rows.push(SettingRow::HitSoundPriorityBd);
            rows.push(SettingRow::RandomMode);
            rows.push(SettingRow::DarkMode);
            rows
        }
        SettingCategory::Drums => {
            let rows: Vec<SettingRow> = vec![
                SettingRow::HhGroup,
                SettingRow::FtGroup,
                SettingRow::CyGroup,
                SettingRow::BdGroup,
                SettingRow::RdPosition,
                SettingRow::GaugeMode,
                SettingRow::DamageLevel,
                SettingRow::Risky,
                SettingRow::AutoAddGage,
                SettingRow::StoicMode,
                SettingRow::WaveDriftCorrection,
                SettingRow::CompactMode,
                SettingRow::RandomSubBox,
                SettingRow::GuitarTimingOffset,
                SettingRow::BassTimingOffset,
            ];
            rows
        }
        SettingCategory::Input => {
            // Input is the union of all three instrument profiles so a
            // user can see every binding in one screen. The Settings
            // overlay's instrument_filter still scopes new bindings.
            let mut rows: Vec<SettingRow> = Vec::with_capacity(11 + 5 + 4 + 6);
            for lane in 0..LANES.len() {
                rows.push(SettingRow::LaneKey(BindingTarget::DrumLane(
                    DrumLane::from_index(lane).expect("valid drum lane index"),
                )));
            }
            for lane in GuitarLane::VISIBLE_LANES {
                rows.push(SettingRow::LaneKey(BindingTarget::GuitarLane(lane)));
            }
            for lane in BassLane::VISIBLE_LANES {
                rows.push(SettingRow::LaneKey(BindingTarget::BassLane(lane)));
            }
            rows.extend(
                SYSTEM_ACTION_SETTINGS_ORDER
                    .iter()
                    .copied()
                    .map(SettingRow::SystemAction),
            );
            rows
        }
        SettingCategory::InputDrums => {
            let mut rows: Vec<SettingRow> = (0..LANES.len())
                .map(|lane| {
                    SettingRow::LaneKey(BindingTarget::DrumLane(
                        DrumLane::from_index(lane).expect("valid drum lane index"),
                    ))
                })
                .collect();
            rows.extend(
                SYSTEM_ACTION_SETTINGS_ORDER
                    .iter()
                    .copied()
                    .map(SettingRow::SystemAction),
            );
            rows
        }
        SettingCategory::InputGuitar => {
            let mut rows: Vec<SettingRow> = GuitarLane::VISIBLE_LANES
                .iter()
                .map(|lane| SettingRow::LaneKey(BindingTarget::GuitarLane(*lane)))
                .collect();
            rows.extend(
                SYSTEM_ACTION_SETTINGS_ORDER
                    .iter()
                    .copied()
                    .map(SettingRow::SystemAction),
            );
            rows
        }
        SettingCategory::InputBass => {
            let mut rows: Vec<SettingRow> = BassLane::VISIBLE_LANES
                .iter()
                .map(|lane| SettingRow::LaneKey(BindingTarget::BassLane(*lane)))
                .collect();
            rows.extend(
                SYSTEM_ACTION_SETTINGS_ORDER
                    .iter()
                    .copied()
                    .map(SettingRow::SystemAction),
            );
            rows
        }
        SettingCategory::Graphics => vec![SettingRow::FpsCap],
        SettingCategory::Debug => vec![
            SettingRow::MetronomeSound,
            SettingRow::UseOsTimer,
            SettingRow::ChipPlayTimeComputeMode,
            SettingRow::WriteScoreIni,
            SettingRow::DebugHud,
        ],
    }
}

pub(crate) fn all_setting_rows() -> Vec<SettingRow> {
    [
        SettingCategory::General,
        SettingCategory::Audio,
        SettingCategory::Gameplay,
        SettingCategory::Drums,
        SettingCategory::Input,
        SettingCategory::InputDrums,
        SettingCategory::InputGuitar,
        SettingCategory::InputBass,
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
                let label = row.label().to_ascii_lowercase();
                let description = row.description().to_ascii_lowercase();
                let category = row.category().label().to_ascii_lowercase();
                label.contains(&search)
                    || description.contains(&search)
                    || category.contains(&search)
            }
        })
        .collect()
}

impl SettingCategory {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            SettingCategory::General => "General",
            SettingCategory::Audio => "Audio",
            SettingCategory::Gameplay => "Gameplay",
            SettingCategory::Drums => "Drums",
            SettingCategory::Input => "Input (all)",
            SettingCategory::InputDrums => "Input (drums)",
            SettingCategory::InputGuitar => "Input (guitar)",
            SettingCategory::InputBass => "Input (bass)",
            SettingCategory::Graphics => "Graphics",
            SettingCategory::Debug => "Debug",
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            SettingCategory::General => SettingCategory::Audio,
            SettingCategory::Audio => SettingCategory::Gameplay,
            SettingCategory::Gameplay => SettingCategory::Drums,
            SettingCategory::Drums => SettingCategory::InputDrums,
            SettingCategory::InputDrums => SettingCategory::InputGuitar,
            SettingCategory::InputGuitar => SettingCategory::InputBass,
            SettingCategory::InputBass => SettingCategory::Input,
            SettingCategory::Input => SettingCategory::Graphics,
            SettingCategory::Graphics => SettingCategory::Debug,
            SettingCategory::Debug => SettingCategory::General,
        }
    }
}

impl SettingRow {
    pub(crate) fn category(&self) -> SettingCategory {
        match self {
            SettingRow::ChartRoot => SettingCategory::General,
            SettingRow::MasterVolume
            | SettingRow::BgmVolume
            | SettingRow::DrumVolume
            | SettingRow::TimingOffset => SettingCategory::Audio,
            SettingRow::Practice
            | SettingRow::SkillMode
            | SettingRow::AutoMode
            | SettingRow::PerLaneAuto(_)
            | SettingRow::LaneSpeed
            | SettingRow::PedalLagTime
            | SettingRow::SongRate
            | SettingRow::PlaySpeedNum
            | SettingRow::PlaySpeedDen
            | SettingRow::SaveScoreIfModifiedPlaySpeed
            | SettingRow::CymbalFree
            | SettingRow::LpMuting
            | SettingRow::DrumHitSound
            | SettingRow::HitSoundPriorityHh
            | SettingRow::HitSoundPriorityTom
            | SettingRow::HitSoundPriorityCymbal
            | SettingRow::HitSoundPriorityBd
            | SettingRow::RandomMode
            | SettingRow::DarkMode => SettingCategory::Gameplay,
            SettingRow::HhGroup
            | SettingRow::FtGroup
            | SettingRow::CyGroup
            | SettingRow::BdGroup
            | SettingRow::RdPosition
            | SettingRow::GaugeMode
            | SettingRow::DamageLevel
            | SettingRow::Risky
            | SettingRow::AutoAddGage
            | SettingRow::StoicMode
            | SettingRow::CompactMode
            | SettingRow::RandomSubBox
            | SettingRow::WaveDriftCorrection
            | SettingRow::GuitarTimingOffset
            | SettingRow::BassTimingOffset => SettingCategory::Drums,
            SettingRow::LaneKey(target) => match target {
                BindingTarget::DrumLane(_) => SettingCategory::InputDrums,
                BindingTarget::GuitarLane(_) => SettingCategory::InputGuitar,
                BindingTarget::BassLane(_) => SettingCategory::InputBass,
                BindingTarget::System(_) => SettingCategory::Input,
            },
            SettingRow::SystemAction(_) => SettingCategory::Input,
            SettingRow::FpsCap => SettingCategory::Graphics,
            SettingRow::MetronomeSound
            | SettingRow::UseOsTimer
            | SettingRow::ChipPlayTimeComputeMode
            | SettingRow::WriteScoreIni
            | SettingRow::DebugHud => SettingCategory::Debug,
        }
    }

    pub(crate) fn is_adjustable(&self) -> bool {
        !matches!(
            self,
            SettingRow::ChartRoot | SettingRow::LaneKey(_) | SettingRow::SystemAction(_)
        )
    }

    pub(crate) fn live_adjustable(&self, active_practice: Option<bool>) -> bool {
        if !self.is_adjustable() {
            return false;
        }
        match self {
            SettingRow::Practice | SettingRow::AutoMode | SettingRow::PerLaneAuto(_) => {
                play_mode_change_allowed_during_play(active_practice)
            }
            SettingRow::SongRate => song_rate_change_allowed_during_play(active_practice),
            _ => true,
        }
    }

    pub(crate) fn is_toggle(&self) -> bool {
        matches!(
            self,
            SettingRow::MetronomeSound
                | SettingRow::LpMuting
                | SettingRow::DrumHitSound
                | SettingRow::CymbalFree
                | SettingRow::UseOsTimer
                | SettingRow::WriteScoreIni
                | SettingRow::DebugHud
                | SettingRow::Practice
                | SettingRow::PerLaneAuto(_)
                | SettingRow::SaveScoreIfModifiedPlaySpeed
                | SettingRow::AutoAddGage
                | SettingRow::StoicMode
                | SettingRow::CompactMode
                | SettingRow::RandomSubBox
                | SettingRow::WaveDriftCorrection
        )
    }

    pub(crate) fn toggle_value(&self, config: &GameConfig) -> bool {
        match self {
            SettingRow::MetronomeSound => config.metronome_sound,
            SettingRow::LpMuting => config.lp_muting,
            SettingRow::DrumHitSound => config.drum_hit_sound,
            SettingRow::CymbalFree => config.cymbal_free,
            SettingRow::UseOsTimer => config.use_os_timer,
            SettingRow::WriteScoreIni => config.write_score_ini,
            SettingRow::DebugHud => config.show_debug_hud,
            SettingRow::Practice => config.practice_song_select,
            SettingRow::PerLaneAuto(lane) => config.per_lane_auto.contains(lane),
            SettingRow::SaveScoreIfModifiedPlaySpeed => config.save_score_if_modified_play_speed,
            SettingRow::AutoAddGage => config.gauge.auto_add_gauge,
            SettingRow::StoicMode => config.stoic_mode,
            SettingRow::CompactMode => config.compact_mode,
            SettingRow::RandomSubBox => config.random_sub_box,
            SettingRow::WaveDriftCorrection => config.wave_drift_correction,
            _ => false,
        }
    }

    pub(crate) fn slider_ratio(&self, config: &GameConfig, mix: &AudioMix) -> Option<f32> {
        match self {
            SettingRow::MasterVolume => Some(mix.master),
            SettingRow::BgmVolume => Some(mix.bgm),
            SettingRow::DrumVolume => Some(mix.drums),
            SettingRow::TimingOffset => Some(((config.timing_offset + 0.5) / 1.0).clamp(0.0, 1.0)),
            SettingRow::GuitarTimingOffset => {
                Some(((config.guitar_offset + 0.5) / 1.0).clamp(0.0, 1.0))
            }
            SettingRow::BassTimingOffset => {
                Some(((config.bass_offset + 0.5) / 1.0).clamp(0.0, 1.0))
            }
            SettingRow::LaneSpeed => Some(
                ((config.lane_speed - MIN_LANE_SPEED) / (MAX_LANE_SPEED - MIN_LANE_SPEED))
                    .clamp(0.0, 1.0),
            ),
            SettingRow::PedalLagTime => {
                Some(((config.pedal_lag_time_ms + 100) as f32 / 200.0).clamp(0.0, 1.0))
            }
            SettingRow::SongRate => Some(
                ((config.song_playback_rate - MIN_SONG_PLAYBACK_RATE)
                    / (MAX_SONG_PLAYBACK_RATE - MIN_SONG_PLAYBACK_RATE))
                    .clamp(0.0, 1.0),
            ),
            _ => None,
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        match self {
            SettingRow::ChartRoot => "Chart root",
            SettingRow::MasterVolume => "Master volume",
            SettingRow::BgmVolume => "BGM volume",
            SettingRow::DrumVolume => "Drum volume",
            SettingRow::Practice => "Practice",
            SettingRow::SkillMode => "Skill mode",
            SettingRow::AutoMode => "Auto mode",
            SettingRow::PerLaneAuto(lane) => match lane {
                DrumLane::Bd => "Auto: BD",
                DrumLane::Sd => "Auto: SD",
                DrumLane::Ft => "Auto: FT",
                DrumLane::Hh => "Auto: HH",
                DrumLane::Lp => "Auto: LP",
                DrumLane::Lt => "Auto: LT",
                DrumLane::Ht => "Auto: HT",
                DrumLane::Cy => "Auto: CY",
                DrumLane::Rd => "Auto: RD",
                DrumLane::Lc => "Auto: LC",
                DrumLane::Lbd => "Auto: LBD",
            },
            SettingRow::LaneSpeed => "Lane speed",
            SettingRow::PedalLagTime => "Pedal lag",
            SettingRow::CymbalFree => "Cymbal free",
            SettingRow::LaneKey(target) => lane_label(target.clone()),
            SettingRow::SystemAction(action) => action.label(),
            SettingRow::TimingOffset => "Drum offset",
            SettingRow::GuitarTimingOffset => "Guitar offset",
            SettingRow::BassTimingOffset => "Bass offset",
            SettingRow::PlaySpeedNum => "Play speed N",
            SettingRow::PlaySpeedDen => "Play speed D",
            SettingRow::SaveScoreIfModifiedPlaySpeed => "Save score @ modified speed",
            SettingRow::SongRate => "Song rate",
            SettingRow::HhGroup => "HH group",
            SettingRow::FtGroup => "FT group",
            SettingRow::CyGroup => "CY group",
            SettingRow::BdGroup => "BD group",
            SettingRow::RdPosition => "RD position",
            SettingRow::DarkMode => "Dark mode",
            SettingRow::RandomMode => "Random",
            SettingRow::GaugeMode => "Gauge",
            SettingRow::Risky => "Risky",
            SettingRow::DamageLevel => "Damage",
            SettingRow::AutoAddGage => "Auto-add gauge",
            SettingRow::StoicMode => "Stoic",
            SettingRow::CompactMode => "Compact mode",
            SettingRow::RandomSubBox => "Random sub-boxes",
            SettingRow::WaveDriftCorrection => "WAV drift correction",
            SettingRow::FpsCap => "FPS cap",
            SettingRow::MetronomeSound => "Metronome sound",
            SettingRow::UseOsTimer => "OS timer",
            SettingRow::ChipPlayTimeComputeMode => "Chip timing",
            SettingRow::WriteScoreIni => "Write score.ini",
            SettingRow::LpMuting => "LP muting",
            SettingRow::DrumHitSound => "Drum hit sound",
            SettingRow::HitSoundPriorityHh => "HH hit priority",
            SettingRow::HitSoundPriorityTom => "Tom hit priority",
            SettingRow::HitSoundPriorityCymbal => "Cymbal hit priority",
            SettingRow::HitSoundPriorityBd => "BD hit priority",
            SettingRow::DebugHud => "Debug HUD",
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            SettingRow::ChartRoot => "Folder scanned for songs. Edit config.ron to change.",
            SettingRow::MasterVolume => "Global output level.",
            SettingRow::BgmVolume => "Song backing track level.",
            SettingRow::DrumVolume => "Drum hit and auto-SE level.",
            SettingRow::Practice => {
                "Top-level mode. Practice skips gauge fail and disables leaderboard submission. Locked at song start."
            }
            SettingRow::SkillMode => {
                "BocuD rank/skill formula mode. New = current DTXManiaNX default; Old = legacy rank formula."
            }
            SettingRow::AutoMode => {
                "Off = no auto. Per-lane = use the per-lane auto config below. All auto = all 10 lanes auto for this run, your per-lane config is preserved."
            }
            SettingRow::PerLaneAuto(_) => {
                "Toggle this lane on to auto-play it (when Auto mode = Per-lane or All auto)."
            }
            SettingRow::LaneSpeed => "Note scroll speed. Applies immediately during play.",
            SettingRow::PedalLagTime => {
                "Extra timing offset for BD/LP/LBD pedal chips in milliseconds."
            }
            SettingRow::CymbalFree => "When enabled, any cymbal input may hit CY/RD/LC notes.",
            SettingRow::LaneKey(_) => {
                "Lane bindings. Enter adds key or MIDI pad. ←/→ select binding. Backspace/Delete removes selected."
            }
            SettingRow::SystemAction(action) => action.description(),
            SettingRow::TimingOffset => {
                "Drum visual/judgement offset in milliseconds. Per-instrument (see also Guitar offset, Bass offset)."
            }
            SettingRow::GuitarTimingOffset => {
                "Guitar visual/judgement offset in milliseconds. Per-instrument calibration (BocuD `nInputAdjustTime` for guitar)."
            }
            SettingRow::BassTimingOffset => {
                "Bass visual/judgement offset in milliseconds. Per-instrument calibration."
            }
            SettingRow::PlaySpeedNum => {
                "Play speed numerator (BocuD `nPlaySpeedNumerator`, default 20). Effective rate = N/D."
            }
            SettingRow::PlaySpeedDen => {
                "Play speed denominator (BocuD `nPlaySpeedDenominator`, default 20)."
            }
            SettingRow::SaveScoreIfModifiedPlaySpeed => {
                "BocuD `bSaveScoreIfModifiedPlaySpeed`. When OFF, runs at non-1.0× play speed are excluded from the best board."
            }
            SettingRow::SongRate => {
                "Song playback rate. Practice mode only during play; always editable from menus."
            }
            SettingRow::HhGroup => {
                "BocuD `EHHGroup`. HH/LC pad grouping policy (all-split / HH-only / LC-only / all-common)."
            }
            SettingRow::FtGroup => "BocuD `EFTGroup`. Tom (LT/FT) grouping policy.",
            SettingRow::CyGroup => "BocuD `ECYGroup`. Cymbal (CY/RD) grouping policy.",
            SettingRow::BdGroup => "BocuD `EBDGroup`. BD/LP/LBD pedal grouping policy.",
            SettingRow::RdPosition => {
                "BocuD `ERDPosition`. Physical lane order for RD and CY (RD-RC vs RC-RD)."
            }
            SettingRow::DarkMode => {
                "BocuD `EDarkMode`. Note visibility policy (off / half / full)."
            }
            SettingRow::RandomMode => {
                "BocuD `ERandomMode` (7 variants: off/mirror/random/super/hyper/master/another). Applied to drum pads."
            }
            SettingRow::GaugeMode => {
                "BocuD gauge difficulty factor index (normal / hard / death / extreme / ex-hard)."
            }
            SettingRow::Risky => "BocuD Risky mode initial miss count (0 = off, 1..=10 = Risky N).",
            SettingRow::DamageLevel => {
                "BocuD `EDamageLevel`. Multiplier on Poor/Miss gauge delta (small=0.5× / normal=1× / high=2×)."
            }
            SettingRow::AutoAddGage => {
                "BocuD `bAutoAddGage`. Auto-played chips contribute positive gauge delta."
            }
            SettingRow::StoicMode => {
                "BocuD `bストイックモード`. Suppresses in-play animations and song-select preimage animation. Wiring in Phase E."
            }
            SettingRow::CompactMode => {
                "BocuD `bCompactMode`. Skip Title stage on startup. Wiring in Phase C-D.6."
            }
            SettingRow::RandomSubBox => {
                "BocuD `bランダムセレクトで子BOXを検索対象とする`. Random song select descends into sub-boxes (Phase F wiring)."
            }
            SettingRow::WaveDriftCorrection => {
                "BocuD `bWave再生位置自動調整機能有効`. Kira already handles long-WAV position correction. User-facing toggle."
            }
            SettingRow::FpsCap => {
                "Frame rate cap (VSync / 60 / 120 / 144 / 240 / Unlimited). Hotkey: F6."
            }
            SettingRow::MetronomeSound => "Play metronome clicks during gameplay.",
            SettingRow::UseOsTimer => {
                "Use OS high-resolution timer for chip scheduling (compat toggle; audio backend support comes later)."
            }
            SettingRow::ChipPlayTimeComputeMode => {
                "DTXManiaNX chip timing mode: Original or Accurate."
            }
            SettingRow::WriteScoreIni => {
                "Write per-chart .score.ini files when score persistence lands."
            }
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

    pub(crate) fn value(&self, config: &GameConfig, mix: &AudioMix) -> String {
        match self {
            SettingRow::LaneKey(target) => {
                keyboard_summary_for_target(&config.bindings, target.clone())
            }
            _ => self.value_with_overlay(config, mix, &SettingsOverlay::default()),
        }
    }

    pub(crate) fn value_with_overlay(
        &self,
        config: &GameConfig,
        mix: &AudioMix,
        overlay: &SettingsOverlay,
    ) -> String {
        match self {
            SettingRow::ChartRoot => config.chart_root.clone(),
            SettingRow::MasterVolume => format!("{:.0}%", mix.master * 100.0),
            SettingRow::BgmVolume => format!("{:.0}%", mix.bgm * 100.0),
            SettingRow::DrumVolume => format!("{:.0}%", mix.drums * 100.0),
            SettingRow::Practice => on_off(config.practice_song_select),
            SettingRow::SkillMode => config.skill_mode.label().to_string(),
            SettingRow::AutoMode => config.auto_mode.label().to_string(),
            SettingRow::PerLaneAuto(lane) => on_off(config.per_lane_auto.contains(lane)),
            SettingRow::HitSoundPriorityHh => config.hit_sound_priority_hh.label().to_string(),
            SettingRow::HitSoundPriorityTom => config.hit_sound_priority_ft.label().to_string(),
            SettingRow::HitSoundPriorityCymbal => config.hit_sound_priority_cy.label().to_string(),
            SettingRow::HitSoundPriorityBd => config.hit_sound_priority_lp.label().to_string(),
            SettingRow::LaneSpeed => format!("{:.2}x", config.lane_speed),
            SettingRow::PedalLagTime => format!("{:+}ms", config.pedal_lag_time_ms),
            SettingRow::CymbalFree => on_off(config.cymbal_free),
            SettingRow::LaneKey(target) => target_bindings_value(
                &config.bindings,
                target.clone(),
                Some(overlay.lane_binding_cursor),
            ),
            SettingRow::SystemAction(action) => {
                system_action_binding_value(&config.bindings, *action)
            }
            SettingRow::TimingOffset => format!("{:+.0}ms", config.timing_offset * 1000.0),
            SettingRow::GuitarTimingOffset => {
                format!("{:+.0}ms", config.guitar_offset * 1000.0)
            }
            SettingRow::BassTimingOffset => {
                format!("{:+.0}ms", config.bass_offset * 1000.0)
            }
            SettingRow::PlaySpeedNum => format!("{}", config.play_speed_num),
            SettingRow::PlaySpeedDen => format!("{}", config.play_speed_den),
            SettingRow::SaveScoreIfModifiedPlaySpeed => {
                on_off(config.save_score_if_modified_play_speed)
            }
            SettingRow::HhGroup => hh_group_label(config.hh_group).to_string(),
            SettingRow::FtGroup => ft_group_label(config.ft_group).to_string(),
            SettingRow::CyGroup => cy_group_label(config.cy_group).to_string(),
            SettingRow::BdGroup => bd_group_label(config.bd_group).to_string(),
            SettingRow::RdPosition => rd_position_label(config.rd_position).to_string(),
            SettingRow::DarkMode => dark_mode_label(config.dark).to_string(),
            SettingRow::RandomMode => random_mode_label(config.random).to_string(),
            SettingRow::GaugeMode => gauge_mode_label(config.gauge.mode).to_string(),
            SettingRow::Risky => {
                if config.gauge.risky_initial == 0 {
                    "off".to_string()
                } else {
                    format!("Risky {}", config.gauge.risky_initial)
                }
            }
            SettingRow::DamageLevel => damage_level_label(config.gauge.damage_level).to_string(),
            SettingRow::AutoAddGage => on_off(config.gauge.auto_add_gauge),
            SettingRow::StoicMode => on_off(config.stoic_mode),
            SettingRow::CompactMode => on_off(config.compact_mode),
            SettingRow::RandomSubBox => on_off(config.random_sub_box),
            SettingRow::WaveDriftCorrection => on_off(config.wave_drift_correction),
            SettingRow::SongRate => format!("{:.2}x", config.song_playback_rate),
            SettingRow::FpsCap => config.fps_cap.label().to_string(),
            SettingRow::MetronomeSound => on_off(config.metronome_sound),
            SettingRow::UseOsTimer => on_off(config.use_os_timer),
            SettingRow::ChipPlayTimeComputeMode => {
                config.chip_play_time_compute_mode.label().to_string()
            }
            SettingRow::WriteScoreIni => on_off(config.write_score_ini),
            SettingRow::LpMuting => on_off(config.lp_muting),
            SettingRow::DrumHitSound => on_off(config.drum_hit_sound),
            SettingRow::DebugHud => on_off(config.show_debug_hud),
        }
    }
}

fn on_off(value: bool) -> String {
    if value { "on".into() } else { "off".into() }
}

fn hh_group_label(g: HHGroup) -> &'static str {
    match g {
        HHGroup::AllSplit => "all split",
        HHGroup::HhOnlySplit => "HH only",
        HHGroup::LcOnlySplit => "LC only",
        HHGroup::AllCommon => "all common",
    }
}

fn ft_group_label(g: FTGroup) -> &'static str {
    match g {
        FTGroup::Split => "split",
        FTGroup::Common => "common",
    }
}

fn cy_group_label(g: CYGroup) -> &'static str {
    match g {
        CYGroup::Split => "split",
        CYGroup::Common => "common",
    }
}

fn lane_label(target: BindingTarget) -> &'static str {
    match target {
        BindingTarget::DrumLane(lane) => LANES[lane.index()].label,
        BindingTarget::GuitarLane(GuitarLane::R) => "Gt R",
        BindingTarget::GuitarLane(GuitarLane::G) => "Gt G",
        BindingTarget::GuitarLane(GuitarLane::B) => "Gt B",
        BindingTarget::GuitarLane(GuitarLane::Y) => "Gt Y",
        BindingTarget::GuitarLane(GuitarLane::P) => "Gt P",
        BindingTarget::GuitarLane(_) => "Gt ?",
        BindingTarget::BassLane(BassLane::R) => "Bs R",
        BindingTarget::BassLane(BassLane::G) => "Bs G",
        BindingTarget::BassLane(BassLane::B) => "Bs B",
        BindingTarget::BassLane(BassLane::P) => "Bs P",
        BindingTarget::BassLane(_) => "Bs ?",
        BindingTarget::System(_) => "Sys",
    }
}

fn bd_group_label(g: BDGroup) -> &'static str {
    match g {
        BDGroup::Split => "split",
        BDGroup::BdAndLp => "BD+LP",
        BDGroup::LpPair => "LP pair",
        BDGroup::BothBd => "both BD",
    }
}

fn rd_position_label(p: RDPosition) -> &'static str {
    match p {
        RDPosition::RdRc => "RD-RC",
        RDPosition::RcRd => "RC-RD",
    }
}

fn dark_mode_label(d: DarkMode) -> &'static str {
    match d {
        DarkMode::Off => "off",
        DarkMode::Half => "half",
        DarkMode::Full => "full",
    }
}

fn random_mode_label(r: RandomMode) -> &'static str {
    r.label()
}

fn gauge_mode_label(m: GaugeMode) -> &'static str {
    match m {
        GaugeMode::Normal => "normal",
        GaugeMode::Hard => "hard",
        GaugeMode::Death => "death",
        GaugeMode::Extreme => "extreme",
        GaugeMode::ExHard => "ex-hard",
    }
}

fn damage_level_label(d: DamageLevel) -> &'static str {
    match d {
        DamageLevel::Small => "small",
        DamageLevel::Normal => "normal",
        DamageLevel::High => "high",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_rows_splits_input_per_instrument() {
        // Per-instrument sub-categories carry the matching target set so
        // the rebind UI can scope a single instrument at a time. The
        // aggregated "Input" category still shows everything.
        let drums = category_rows(SettingCategory::InputDrums);
        let guitar = category_rows(SettingCategory::InputGuitar);
        let bass = category_rows(SettingCategory::InputBass);
        let all = category_rows(SettingCategory::Input);

        let drum_targets: Vec<&BindingTarget> = drums
            .iter()
            .filter_map(|row| match row {
                SettingRow::LaneKey(t) => Some(t),
                _ => None,
            })
            .collect();
        assert!(drum_targets.contains(&&BindingTarget::DrumLane(DrumLane::Bd)));
        assert!(drum_targets.contains(&&BindingTarget::DrumLane(DrumLane::Lbd)));
        assert!(
            !drum_targets
                .iter()
                .any(|t| matches!(t, BindingTarget::GuitarLane(_)))
        );
        assert!(
            !drum_targets
                .iter()
                .any(|t| matches!(t, BindingTarget::BassLane(_)))
        );

        let guitar_targets: Vec<&BindingTarget> = guitar
            .iter()
            .filter_map(|row| match row {
                SettingRow::LaneKey(t) => Some(t),
                _ => None,
            })
            .collect();
        assert!(guitar_targets.contains(&&BindingTarget::GuitarLane(GuitarLane::R)));
        assert!(guitar_targets.contains(&&BindingTarget::GuitarLane(GuitarLane::P)));
        assert_eq!(guitar_targets.len(), GuitarLane::VISIBLE_LANES.len());

        let bass_targets: Vec<&BindingTarget> = bass
            .iter()
            .filter_map(|row| match row {
                SettingRow::LaneKey(t) => Some(t),
                _ => None,
            })
            .collect();
        assert!(bass_targets.contains(&&BindingTarget::BassLane(BassLane::R)));
        assert_eq!(bass_targets.len(), BassLane::VISIBLE_LANES.len());

        // Aggregated "Input" carries every per-instrument lane plus
        // system actions; the row count must equal the sum of the
        // per-instrument lane counts plus the system action count.
        let all_lane_count = all
            .iter()
            .filter(|row| matches!(row, SettingRow::LaneKey(_)))
            .count();
        assert_eq!(
            all_lane_count,
            LANES.len() + GuitarLane::VISIBLE_LANES.len() + BassLane::VISIBLE_LANES.len()
        );
    }

    #[test]
    fn setting_row_category_routes_per_instrument() {
        assert_eq!(
            SettingRow::LaneKey(BindingTarget::DrumLane(DrumLane::Bd)).category(),
            SettingCategory::InputDrums
        );
        assert_eq!(
            SettingRow::LaneKey(BindingTarget::GuitarLane(GuitarLane::R)).category(),
            SettingCategory::InputGuitar
        );
        assert_eq!(
            SettingRow::LaneKey(BindingTarget::BassLane(BassLane::R)).category(),
            SettingCategory::InputBass
        );
        assert_eq!(
            SettingRow::SystemAction(SystemAction::PauseToggle).category(),
            SettingCategory::Input
        );
    }

    #[test]
    fn setting_category_next_cycles_all_input_subcategories() {
        // The "next" chain must reach every Input subcategory so Tab
        // navigation in the overlay surfaces them all.
        let mut cat = SettingCategory::Drums;
        let mut visited = vec![cat];
        for _ in 0..12 {
            cat = cat.next();
            visited.push(cat);
        }
        assert!(visited.contains(&SettingCategory::InputDrums));
        assert!(visited.contains(&SettingCategory::InputGuitar));
        assert!(visited.contains(&SettingCategory::InputBass));
        assert!(visited.contains(&SettingCategory::Input));
    }

    #[test]
    fn bocud_boolean_rows_render_as_toggles() {
        let config = GameConfig {
            save_score_if_modified_play_speed: true,
            gauge: crate::config::GaugeConfig {
                auto_add_gauge: true,
                ..Default::default()
            },
            stoic_mode: true,
            compact_mode: true,
            random_sub_box: true,
            wave_drift_correction: false,
            ..Default::default()
        };

        let rows = [
            (SettingRow::SaveScoreIfModifiedPlaySpeed, true),
            (SettingRow::AutoAddGage, true),
            (SettingRow::StoicMode, true),
            (SettingRow::CompactMode, true),
            (SettingRow::RandomSubBox, true),
            (SettingRow::WaveDriftCorrection, false),
        ];

        for (row, expected) in rows {
            assert!(row.is_toggle(), "{} should draw as a toggle", row.label());
            assert_eq!(row.toggle_value(&config), expected);
        }
    }

    #[test]
    fn per_instrument_offsets_have_slider_ratios() {
        let config = GameConfig {
            guitar_offset: 0.25,
            bass_offset: -0.25,
            ..Default::default()
        };
        let mix = AudioMix::from_config(&config);

        assert_eq!(
            SettingRow::GuitarTimingOffset.slider_ratio(&config, &mix),
            Some(0.75)
        );
        assert_eq!(
            SettingRow::BassTimingOffset.slider_ratio(&config, &mix),
            Some(0.25)
        );
    }
}
