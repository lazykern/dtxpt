use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use bevy_framepace::Limiter;
use dtxpt::input::bindings::{InputBindingConfig, PlayMode, default_input_bindings};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Frame rate cap. Replaces the old `vsync: bool` toggle.
///
/// - `Vsync`: cap to monitor refresh via vsync. Default; matches DTXMania osu!lazer.
/// - `Cap60/120/144/240`: hard cap via `WinitSettings::continuous(max_wait)`.
///   `Immediate` present mode, so the GPU isn't blocked by vsync when the cap
///   is below monitor refresh.
/// - `Unlimited`: no cap, no vsync, `Immediate` present. Lowest input latency
///   but allows tearing. For rhythm games the judgement line is the focus so
///   tearing artefacts are acceptable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FpsCap {
    Vsync,
    Cap60,
    Cap120,
    Cap144,
    Cap240,
    Unlimited,
}

impl Default for FpsCap {
    fn default() -> Self {
        Self::Vsync
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitSoundPriority {
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

impl Default for HitSoundPriority {
    fn default() -> Self {
        Self::ChipOverPad
    }
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
    pub timing_offset: f32,
    pub song_playback_rate: f32,
    pub play_mode: PlayMode,
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
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            version: 10,
            chart_root: "charts".into(),
            last_chart_path: String::new(),
            preferred_difficulty: String::new(),
            master_volume: 0.8,
            bgm_volume: 1.0,
            drum_volume: 1.0,
            lane_speed: 1.0,
            timing_offset: 0.0,
            song_playback_rate: 1.0,
            play_mode: PlayMode::Normal,
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
        assert!(matches!(FpsCap::Vsync.limiter(), bevy_framepace::Limiter::Auto));
        assert!(matches!(FpsCap::Unlimited.limiter(), bevy_framepace::Limiter::Off));
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
                    let diff = if d > expected { d - expected } else { expected - d };
                    assert!(diff < Duration::from_micros(1), "{:?} vs {:?}", d, expected);
                }
                other => panic!("expected Manual, got {:?}", other),
            }
        }
    }
}
