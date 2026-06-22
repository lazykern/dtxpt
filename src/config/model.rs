use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use bevy_framepace::Limiter;
use dtxpt::chart::ChipPlayTimeComputeMode;
use dtxpt::input::bindings::{DrumLane, InputBindingConfig, default_input_bindings};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::time::Duration;

use crate::gameplay::mods::AutoMode;

fn default_true() -> bool {
    true
}

/// Frame rate cap. Replaces the old `vsync: bool` toggle.
///
/// - `Vsync`: cap to monitor refresh via vsync. Default; matches DTXMania osu!lazer.
/// - `Cap60/120/144/240`: hard cap via `WinitSettings::continuous(max_wait)`.
///   `Immediate` present mode, so the GPU isn't blocked by vsync when the cap
///   is below monitor refresh.
/// - `Unlimited`: no cap, no vsync, `Immediate` present. Lowest input latency
///   but allows tearing. For rhythm games the judgement line is the focus so
///   tearing artefacts are acceptable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FpsCap {
    #[default]
    Vsync,
    Cap60,
    Cap120,
    Cap144,
    Cap240,
    Unlimited,
}

impl FpsCap {
    pub fn next(self) -> Self {
        match self {
            Self::Vsync => Self::Cap60,
            Self::Cap60 => Self::Cap120,
            Self::Cap120 => Self::Cap144,
            Self::Cap144 => Self::Cap240,
            Self::Cap240 => Self::Unlimited,
            Self::Unlimited => Self::Vsync,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vsync => "VSync",
            Self::Cap60 => "60",
            Self::Cap120 => "120",
            Self::Cap144 => "144",
            Self::Cap240 => "240",
            Self::Unlimited => "Unlimited",
        }
    }

    /// Frame interval for hard caps. `Vsync` and `Unlimited` have no fixed
    /// interval and use `WinitSettings::game_app_mode()`.
    #[allow(dead_code)]
    pub fn frame_duration(self) -> Option<Duration> {
        match self {
            Self::Cap60 => Some(Duration::from_secs_f64(1.0 / 60.0)),
            Self::Cap120 => Some(Duration::from_secs_f64(1.0 / 120.0)),
            Self::Cap144 => Some(Duration::from_secs_f64(1.0 / 144.0)),
            Self::Cap240 => Some(Duration::from_secs_f64(1.0 / 240.0)),
            Self::Vsync | Self::Unlimited => None,
        }
    }

    pub fn winit_settings(self) -> WinitSettings {
        // Frame pacing is handled by `bevy_framepace`, which sleeps the
        // main thread at the start of the event loop to enforce the cap.
        // WinitSettings just controls when Update fires within the loop.
        // Continuous lets Update run as fast as the loop allows, then
        // bevy_framepace throttles the next iteration.
        WinitSettings::continuous()
    }

    /// Frame limiter for `bevy_framepace`. Caps the effective frame rate.
    /// - Vsync: monitor refresh (Auto, dynamically updates on monitor change)
    /// - Cap*: hard cap (Manual)
    /// - Unlimited: no cap (Off)
    pub fn limiter(self) -> Limiter {
        match self {
            Self::Vsync => Limiter::Auto,
            Self::Cap60 => Limiter::from_framerate(60.0),
            Self::Cap120 => Limiter::from_framerate(120.0),
            Self::Cap144 => Limiter::from_framerate(144.0),
            Self::Cap240 => Limiter::from_framerate(240.0),
            Self::Unlimited => Limiter::Off,
        }
    }

    pub fn present_mode(self) -> PresentMode {
        match self {
            Self::Vsync => PresentMode::AutoVsync,
            // Hard caps use Immediate so the GPU isn't blocked by vsync when
            // the cap is below monitor refresh. AutoNoVsync is capped to
            // display rate internally on some platforms; Immediate is the
            // unbuffered single-present and matches the existing no-vsync path.
            Self::Cap60 | Self::Cap120 | Self::Cap144 | Self::Cap240 | Self::Unlimited => {
                PresentMode::Immediate
            }
        }
    }

    pub fn has_vsync(self) -> bool {
        matches!(self, Self::Vsync)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitSoundPriority {
    #[default]
    ChipOverPad,
    PadOverChip,
}

impl HitSoundPriority {
    pub fn label(self) -> &'static str {
        match self {
            Self::ChipOverPad => "chip",
            Self::PadOverChip => "pad",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::ChipOverPad => Self::PadOverChip,
            Self::PadOverChip => Self::ChipOverPad,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillMode {
    Old,
    #[default]
    New,
}

impl SkillMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Old => "Old",
            Self::New => "New",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Old => Self::New,
            Self::New => Self::Old,
        }
    }
}

/// HH/FT/CY/BD grouping rules per BocuD `EHHGroup`/`EFTGroup`/`ECYGroup`/
/// `EBDGroup`. Controls how pads on the same physical input resolve into
/// distinct lanes. Drum-only (no equivalent for guitar/bass).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HHGroup {
    /// All four HH states (HH close/open, LC open/close) are separate.
    #[default]
    AllSplit,
    /// HH close/open are separate, LC is grouped with HH.
    HhOnlySplit,
    /// LC open/close are separate, HH is grouped with LC.
    LcOnlySplit,
    /// HH close/open and LC open/close are all the same lane.
    AllCommon,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FTGroup {
    #[default]
    Split,
    Common,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CYGroup {
    #[default]
    Split,
    Common,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BDGroup {
    /// Separate lanes for BD / LP / LBD.
    #[default]
    Split,
    /// BD and LP share a lane; LBD is separate.
    BdAndLp,
    /// LBD and LP share a lane; BD is separate.
    LpPair,
    /// BD and LP both map to BD; LBD is separate.
    BothBd,
}

/// Physical order of the ride and crash lanes. BocuD `ERDPosition`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RDPosition {
    #[default]
    RdRc,
    RcRd,
}

/// BocuD `EDarkMode`. Note visibility policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DarkMode {
    #[default]
    Off,
    Half,
    Full,
}

impl DarkMode {
    /// Cycle through the three Dark variants. Used by the song-select
    /// quick-config popup (`F5`).
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Half,
            Self::Half => Self::Full,
            Self::Full => Self::Off,
        }
    }
}

/// Damage level applied to gauge deltas on Poor/Miss. BocuD `EDamageLevel`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageLevel {
    #[default]
    Normal,
    Small,
    High,
}

/// Gauge mode per chart difficulty. BocuD `fGaugeFactor[5,2]`. Drum-only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GaugeMode {
    #[default]
    Normal,
    Hard,
    Death,
    Extreme,
    ExHard,
}

/// BocuD `ERandomMode`. Applied to pads (drums) or strings (guitar/bass).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RandomMode {
    #[default]
    Off,
    Mirror,
    Random,
    SuperRandom,
    HyperRandom,
    MasterRandom,
    AnotherRandom,
}

impl RandomMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Mirror => "mirror",
            Self::Random => "random",
            Self::SuperRandom => "super",
            Self::HyperRandom => "hyper",
            Self::MasterRandom => "master",
            Self::AnotherRandom => "another",
        }
    }
}

/// Compact gauge model controls. `risky_initial` of 0 disables Risky.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GaugeConfig {
    pub mode: GaugeMode,
    pub damage_level: DamageLevel,
    pub risky_initial: u8,
    pub auto_add_gauge: bool,
}

fn default_play_speed_num() -> u32 {
    20
}

fn default_play_speed_den() -> u32 {
    20
}

#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GameConfig {
    pub version: u32,
    pub chart_root: String,
    #[serde(default)]
    pub last_chart_path: String,
    #[serde(default)]
    pub preferred_difficulty: String,
    pub master_volume: f64,
    pub bgm_volume: f64,
    pub drum_volume: f64,
    pub lane_speed: f32,
    /// Drum timing offset in seconds. Equivalent to the legacy global
    /// `timing_offset`. Kept as the authoritative drum field; guitar
    /// and bass each have their own `*_offset` field below.
    pub timing_offset: f32,
    /// Guitar timing offset in seconds (BocuD `nInputAdjustTime` for
    /// guitar; per-instrument so guitar and bass can be tuned
    /// independently). Default 0.0; `#[serde(default)]` for v14 RON
    /// upgrade.
    #[serde(default)]
    pub guitar_offset: f32,
    /// Bass timing offset in seconds. Same semantics as
    /// `guitar_offset`.
    #[serde(default)]
    pub bass_offset: f32,
    pub song_playback_rate: f32,
    #[serde(default)]
    pub skill_mode: SkillMode,
    #[serde(default)]
    pub pedal_lag_time_ms: i32,
    #[serde(default)]
    pub cymbal_free: bool,
    #[serde(default = "default_true")]
    pub write_score_ini: bool,
    #[serde(default)]
    pub chip_play_time_compute_mode: ChipPlayTimeComputeMode,
    #[serde(default)]
    pub use_os_timer: bool,
    /// User's saved per-lane auto config. Persistent. AutoMode decides
    /// how this is used at run start.
    #[serde(default)]
    pub per_lane_auto: BTreeSet<DrumLane>,
    /// Per-song auto picker. Persistent in config (so user's last pick
    /// is remembered). Resolved with `per_lane_auto` at run start to
    /// produce the run's effective `active_mods.auto_lanes`.
    #[serde(default)]
    pub auto_mode: AutoMode,
    /// Top-level mode toggle. Picked in song-select (or settings).
    /// Persistent in config. Applied to `RunState.practice` at run start.
    #[serde(default)]
    pub practice_song_select: bool,
    pub bindings: Vec<InputBindingConfig>,
    #[serde(default, alias = "lane_keys", skip_serializing)]
    pub legacy_lane_keys: Option<[String; 10]>,
    pub fps_cap: FpsCap,
    pub metronome_sound: bool,
    pub lp_muting: bool,
    pub drum_hit_sound: bool,
    pub hit_sound_priority_hh: HitSoundPriority,
    pub hit_sound_priority_ft: HitSoundPriority,
    pub hit_sound_priority_cy: HitSoundPriority,
    pub hit_sound_priority_lp: HitSoundPriority,
    pub show_debug_hud: bool,
    /// BocuD `bストイックモード`. Suppresses in-play animations and
    /// song-select preimage animation. Drum-only visual.
    #[serde(default)]
    pub stoic_mode: bool,
    /// BocuD `bランダムセレクトで子BOXを検索対象とする`. Random song select
    /// descends into sub-boxes (recursive). Phase F wiring.
    #[serde(default)]
    pub random_sub_box: bool,
    /// BocuD `bCompactMode`. Skip Title stage on startup.
    #[serde(default)]
    pub compact_mode: bool,
    /// BocuD `bWave再生位置自動調整機能有効`. Kira already handles long-WAV
    /// position correction; this is the user-facing toggle.
    #[serde(default = "default_true")]
    pub wave_drift_correction: bool,
    #[serde(default)]
    pub hh_group: HHGroup,
    #[serde(default)]
    pub ft_group: FTGroup,
    #[serde(default)]
    pub cy_group: CYGroup,
    #[serde(default)]
    pub bd_group: BDGroup,
    #[serde(default)]
    pub rd_position: RDPosition,
    #[serde(default)]
    pub dark: DarkMode,
    #[serde(default)]
    pub random: RandomMode,
    #[serde(default)]
    pub gauge: GaugeConfig,
    /// BocuD `nPlaySpeedNumerator/Denominator` (default 20/20 = 1.0×).
    /// Replaces single `f32` rate in score.ini codec.
    #[serde(default = "default_play_speed_num")]
    pub play_speed_num: u32,
    #[serde(default = "default_play_speed_den")]
    pub play_speed_den: u32,
    /// BocuD `bSaveScoreIfModifiedPlaySpeed`. When OFF, runs at non-1.0×
    /// play speed are excluded from the best board.
    #[serde(default)]
    pub save_score_if_modified_play_speed: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            version: 15,
            chart_root: "charts".into(),
            last_chart_path: String::new(),
            preferred_difficulty: String::new(),
            master_volume: 0.8,
            bgm_volume: 1.0,
            drum_volume: 1.0,
            lane_speed: 1.0,
            timing_offset: 0.0,
            guitar_offset: 0.0,
            bass_offset: 0.0,
            song_playback_rate: 1.0,
            skill_mode: SkillMode::default(),
            pedal_lag_time_ms: 0,
            cymbal_free: false,
            write_score_ini: true,
            chip_play_time_compute_mode: ChipPlayTimeComputeMode::default(),
            use_os_timer: false,
            per_lane_auto: BTreeSet::new(),
            auto_mode: AutoMode::PerLane,
            practice_song_select: false,
            bindings: default_input_bindings(),
            legacy_lane_keys: None,
            metronome_sound: true,
            lp_muting: true,
            drum_hit_sound: true,
            hit_sound_priority_hh: HitSoundPriority::ChipOverPad,
            hit_sound_priority_ft: HitSoundPriority::ChipOverPad,
            hit_sound_priority_cy: HitSoundPriority::ChipOverPad,
            hit_sound_priority_lp: HitSoundPriority::ChipOverPad,
            show_debug_hud: false,
            fps_cap: FpsCap::default(),
            stoic_mode: false,
            random_sub_box: false,
            compact_mode: false,
            wave_drift_correction: true,
            hh_group: HHGroup::default(),
            ft_group: FTGroup::default(),
            cy_group: CYGroup::default(),
            bd_group: BDGroup::default(),
            rd_position: RDPosition::default(),
            dark: DarkMode::default(),
            random: RandomMode::default(),
            gauge: GaugeConfig::default(),
            play_speed_num: 20,
            play_speed_den: 20,
            save_score_if_modified_play_speed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_cap_next_cycles_through_all_variants() {
        let order = [
            FpsCap::Vsync,
            FpsCap::Cap60,
            FpsCap::Cap120,
            FpsCap::Cap144,
            FpsCap::Cap240,
            FpsCap::Unlimited,
        ];
        let mut current = FpsCap::Vsync;
        for expected in &order[1..] {
            current = current.next();
            assert_eq!(&current, expected);
        }
        current = current.next();
        assert_eq!(current, FpsCap::Vsync);
    }

    #[test]
    fn fps_cap_frame_duration_only_for_explicit_caps() {
        assert!(FpsCap::Vsync.frame_duration().is_none());
        assert!(FpsCap::Unlimited.frame_duration().is_none());
        assert!(FpsCap::Cap60.frame_duration().is_some());
        assert!(FpsCap::Cap120.frame_duration().is_some());
        assert!(FpsCap::Cap144.frame_duration().is_some());
        assert!(FpsCap::Cap240.frame_duration().is_some());
    }

    #[test]
    fn fps_cap_present_mode_vsync_only_for_vsync() {
        assert_eq!(FpsCap::Vsync.present_mode(), PresentMode::AutoVsync);
        assert_eq!(FpsCap::Cap60.present_mode(), PresentMode::Immediate);
        assert_eq!(FpsCap::Unlimited.present_mode(), PresentMode::Immediate);
        assert!(FpsCap::Vsync.has_vsync());
        assert!(!FpsCap::Cap60.has_vsync());
        assert!(!FpsCap::Unlimited.has_vsync());
    }

    #[test]
    fn fps_cap_limiter_matches_oscillation_expectations() {
        // Vsync -> monitor refresh (Auto); Cap* -> Manual with right duration;
        // Unlimited -> Off.
        assert!(matches!(
            FpsCap::Vsync.limiter(),
            bevy_framepace::Limiter::Auto
        ));
        assert!(matches!(
            FpsCap::Unlimited.limiter(),
            bevy_framepace::Limiter::Off
        ));
        for (cap, hz) in [
            (FpsCap::Cap60, 60.0),
            (FpsCap::Cap120, 120.0),
            (FpsCap::Cap144, 144.0),
            (FpsCap::Cap240, 240.0),
        ] {
            match cap.limiter() {
                bevy_framepace::Limiter::Manual(d) => {
                    let expected = Duration::from_secs_f64(1.0 / hz);
                    // 1us tolerance for float rounding
                    let diff = d.abs_diff(expected);
                    assert!(diff < Duration::from_micros(1), "{:?} vs {:?}", d, expected);
                }
                other => panic!("expected Manual, got {:?}", other),
            }
        }
    }

    #[test]
    fn v12_config_roundtrips_through_ron() {
        let mut per_lane = BTreeSet::new();
        per_lane.insert(dtxpt::input::bindings::DrumLane::Bd);
        per_lane.insert(dtxpt::input::bindings::DrumLane::Hh);
        let cfg = GameConfig {
            per_lane_auto: per_lane,
            auto_mode: crate::gameplay::mods::AutoMode::AllAuto,
            practice_song_select: true,
            ..GameConfig::default()
        };
        let text = ron::ser::to_string(&cfg).expect("serialize");
        let restored: GameConfig = ron::de::from_str(&text).expect("parse");
        assert_eq!(restored.per_lane_auto, cfg.per_lane_auto);
        assert_eq!(restored.auto_mode, cfg.auto_mode);
        assert_eq!(restored.practice_song_select, cfg.practice_song_select);
        assert_eq!(restored.skill_mode, SkillMode::New);
        assert_eq!(
            restored.chip_play_time_compute_mode,
            ChipPlayTimeComputeMode::Accurate
        );
        assert!(restored.write_score_ini);
        assert_eq!(restored.version, 15);
    }

    #[test]
    fn v15_default_is_bumped() {
        assert_eq!(GameConfig::default().version, 15);
    }

    #[test]
    fn v14_new_fields_default_to_bocud_safe_values() {
        let cfg = GameConfig::default();
        assert!(!cfg.stoic_mode);
        assert!(!cfg.random_sub_box);
        assert!(!cfg.compact_mode);
        assert!(cfg.wave_drift_correction);
        assert_eq!(cfg.hh_group, HHGroup::AllSplit);
        assert_eq!(cfg.ft_group, FTGroup::Split);
        assert_eq!(cfg.cy_group, CYGroup::Split);
        assert_eq!(cfg.bd_group, BDGroup::Split);
        assert_eq!(cfg.rd_position, RDPosition::RdRc);
        assert_eq!(cfg.dark, DarkMode::Off);
        assert_eq!(cfg.random, RandomMode::Off);
        assert_eq!(cfg.gauge.mode, GaugeMode::Normal);
        assert_eq!(cfg.gauge.damage_level, DamageLevel::Normal);
        assert_eq!(cfg.gauge.risky_initial, 0);
        assert!(!cfg.gauge.auto_add_gauge);
        assert_eq!(cfg.play_speed_num, 20);
        assert_eq!(cfg.play_speed_den, 20);
        assert!(!cfg.save_score_if_modified_play_speed);
    }

    #[test]
    fn v13_config_roundtrips_into_v14_with_defaults() {
        // A v13 RON config (no v14 fields) should deserialize into a v14
        // GameConfig with all new fields defaulted. This is the migration
        // path for legacy users.
        let v13_text = r#"(
            version: 13,
            chart_root: "charts",
            master_volume: 0.7,
            bgm_volume: 0.9,
            drum_volume: 0.8,
            lane_speed: 1.0,
            timing_offset: 0.0,
            song_playback_rate: 1.0,
            bindings: [],
            fps_cap: Vsync,
            metronome_sound: true,
            lp_muting: true,
            drum_hit_sound: true,
            hit_sound_priority_hh: ChipOverPad,
            hit_sound_priority_ft: ChipOverPad,
            hit_sound_priority_cy: ChipOverPad,
        )"#;
        let cfg: GameConfig = ron::de::from_str(v13_text).expect("parse v13");
        assert_eq!(cfg.version, 13);
        // New v14 fields all default.
        assert!(!cfg.stoic_mode);
        assert!(!cfg.compact_mode);
        assert!(cfg.wave_drift_correction);
        assert_eq!(cfg.hh_group, HHGroup::AllSplit);
        assert_eq!(cfg.gauge.mode, GaugeMode::Normal);
        assert_eq!(cfg.play_speed_num, 20);
        assert_eq!(cfg.play_speed_den, 20);
        // Per-instrument offsets (v15) default to 0.0.
        assert_eq!(cfg.guitar_offset, 0.0);
        assert_eq!(cfg.bass_offset, 0.0);
        // Existing fields preserved.
        assert_eq!(cfg.chart_root, "charts");
        assert!((cfg.master_volume - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn per_instrument_offsets_round_trip_through_ron() {
        let cfg = GameConfig {
            timing_offset: 0.025, // 25ms drum offset
            guitar_offset: 0.012, // 12ms guitar offset
            bass_offset: -0.008,  // -8ms bass offset (early)
            ..GameConfig::default()
        };
        let text = ron::ser::to_string(&cfg).expect("serialize");
        let restored: GameConfig = ron::de::from_str(&text).expect("parse");
        assert!((restored.timing_offset - 0.025).abs() < f32::EPSILON);
        assert!((restored.guitar_offset - 0.012).abs() < f32::EPSILON);
        assert!((restored.bass_offset + 0.008).abs() < f32::EPSILON);
    }

    #[test]
    fn v14_random_mode_covers_bocud_variants() {
        // 7 variants: Off, Mirror, Random, SuperRandom, HyperRandom,
        // MasterRandom, AnotherRandom (matches BocuD `ERandomMode`).
        let variants = [
            RandomMode::Off,
            RandomMode::Mirror,
            RandomMode::Random,
            RandomMode::SuperRandom,
            RandomMode::HyperRandom,
            RandomMode::MasterRandom,
            RandomMode::AnotherRandom,
        ];
        assert_eq!(variants.len(), 7);
        // Round-trip through RON.
        for mode in variants {
            let text = ron::ser::to_string(&mode).expect("serialize");
            let restored: RandomMode = ron::de::from_str(&text).expect("parse");
            assert_eq!(restored, mode);
        }
    }

    #[test]
    fn v15_settings_overlay_extras_round_trip() {
        // BocuD config fields added in v15: GaugeMode (5 variants),
        // DamageLevel (3 variants), HHGroup (4 variants), FTGroup/CYGroup
        // (2 each), BDGroup (4 variants), RDPosition (2 variants),
        // DarkMode (3 variants), play_speed_num/den u32, Risky u32,
        // save_score_if_modified_play_speed bool, stoic_mode bool,
        // compact_mode bool, random_sub_box bool, wave_drift_correction bool.
        let cfg = GameConfig {
            gauge: GaugeConfig {
                mode: GaugeMode::ExHard,
                risky_initial: 3,
                damage_level: DamageLevel::High,
                auto_add_gauge: true,
            },
            hh_group: HHGroup::HhOnlySplit,
            ft_group: FTGroup::Common,
            cy_group: CYGroup::Common,
            bd_group: BDGroup::LpPair,
            rd_position: RDPosition::RcRd,
            dark: DarkMode::Full,
            random: RandomMode::MasterRandom,
            play_speed_num: 22,
            play_speed_den: 20,
            save_score_if_modified_play_speed: false,
            stoic_mode: true,
            compact_mode: true,
            random_sub_box: true,
            wave_drift_correction: false,
            ..GameConfig::default()
        };
        let text = ron::ser::to_string(&cfg).expect("serialize");
        let restored: GameConfig = ron::de::from_str(&text).expect("parse");
        assert_eq!(restored.gauge.mode, GaugeMode::ExHard);
        assert_eq!(restored.gauge.risky_initial, 3);
        assert_eq!(restored.gauge.damage_level, DamageLevel::High);
        assert!(restored.gauge.auto_add_gauge);
        assert_eq!(restored.hh_group, HHGroup::HhOnlySplit);
        assert_eq!(restored.ft_group, FTGroup::Common);
        assert_eq!(restored.cy_group, CYGroup::Common);
        assert_eq!(restored.bd_group, BDGroup::LpPair);
        assert_eq!(restored.rd_position, RDPosition::RcRd);
        assert_eq!(restored.dark, DarkMode::Full);
        assert_eq!(restored.random, RandomMode::MasterRandom);
        assert_eq!(restored.play_speed_num, 22);
        assert_eq!(restored.play_speed_den, 20);
        assert!(!restored.save_score_if_modified_play_speed);
        assert!(restored.stoic_mode);
        assert!(restored.compact_mode);
        assert!(restored.random_sub_box);
        assert!(!restored.wave_drift_correction);
    }
}
