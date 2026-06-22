use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::input::lanes::{
    LANE_BD, LANE_CY, LANE_FT, LANE_HH, LANE_HT, LANE_LBD, LANE_LC, LANE_LP, LANE_LT, LANE_RD,
    LANE_SD,
};

pub const LANE_COUNT: usize = 11;

pub const DEFAULT_LANE_KEY_NAMES: [&str; LANE_COUNT] = [
    "KeyA",
    "KeyS",
    "KeyD",
    "KeyF",
    "KeyL",
    "KeyG",
    "KeyH",
    "KeyJ",
    "KeyK",
    "Semicolon",
    "KeyZ",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DrumLane {
    Bd,
    Sd,
    Ft,
    Hh,
    Lp,
    Lt,
    Ht,
    Cy,
    Rd,
    Lc,
    Lbd,
}

impl DrumLane {
    pub const ALL: [Self; LANE_COUNT] = [
        Self::Bd,
        Self::Sd,
        Self::Ft,
        Self::Hh,
        Self::Lp,
        Self::Lt,
        Self::Ht,
        Self::Cy,
        Self::Rd,
        Self::Lc,
        Self::Lbd,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::Bd => LANE_BD,
            Self::Sd => LANE_SD,
            Self::Ft => LANE_FT,
            Self::Hh => LANE_HH,
            Self::Lp => LANE_LP,
            Self::Lt => LANE_LT,
            Self::Ht => LANE_HT,
            Self::Cy => LANE_CY,
            Self::Rd => LANE_RD,
            Self::Lc => LANE_LC,
            Self::Lbd => LANE_LBD,
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        Some(match index {
            LANE_BD => Self::Bd,
            LANE_SD => Self::Sd,
            LANE_FT => Self::Ft,
            LANE_HH => Self::Hh,
            LANE_LP => Self::Lp,
            LANE_LT => Self::Lt,
            LANE_HT => Self::Ht,
            LANE_CY => Self::Cy,
            LANE_RD => Self::Rd,
            LANE_LC => Self::Lc,
            LANE_LBD => Self::Lbd,
            _ => return None,
        })
    }
}

/// Per-binding instrument tag. Determines which instrument gameplay
/// the binding applies to. Drum bindings stay default for legacy
/// configs (v13 and earlier did not have this field).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BindingInstrument {
    #[default]
    Drums,
    Guitar,
    Bass,
}

impl BindingInstrument {
    pub const ALL: [Self; 3] = [Self::Drums, Self::Guitar, Self::Bass];

    pub fn label(self) -> &'static str {
        match self {
            Self::Drums => "Drums",
            Self::Guitar => "Guitar",
            Self::Bass => "Bass",
        }
    }
}

/// Guitar lanes per BocuD `EChannel` (5 visible lanes + open + control).
/// Channels 0x20..0x27, 0x31..0x38, 0x93..0x9A, 0x9B..0xA2, 0x28..0x2C,
/// 0x2C=44 (long note), 0xBA (NoChip), 0xC0.. (NoChip variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GuitarLane {
    Open,
    R,
    G,
    B,
    Y,
    P,
    Pick,
    Decide,
    Wail,
}

impl GuitarLane {
    pub const VISIBLE_LANES: [Self; 5] = [Self::R, Self::G, Self::B, Self::Y, Self::P];

    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::R => "R",
            Self::G => "G",
            Self::B => "B",
            Self::Y => "Y",
            Self::P => "P",
            Self::Pick => "Pick",
            Self::Decide => "Decide",
            Self::Wail => "Wail",
        }
    }
}

/// Bass lanes per BocuD `EChannel` (4 visible lanes + open + control).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BassLane {
    Open,
    R,
    G,
    B,
    P,
    Pick,
    Decide,
    Wail,
}

impl BassLane {
    pub const VISIBLE_LANES: [Self; 4] = [Self::R, Self::G, Self::B, Self::P];

    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::R => "R",
            Self::G => "G",
            Self::B => "B",
            Self::P => "P",
            Self::Pick => "Pick",
            Self::Decide => "Decide",
            Self::Wail => "Wail",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemAction {
    ToggleSettings,
    PauseToggle,
    RestartChart,
    SeekForward,
    SeekBackward,
    SeekToPreviousMeasure,
    SeekToNextMeasure,
    IncreaseTimingOffset,
    DecreaseTimingOffset,
    ResetTimingOffset,
    IncreaseLaneSpeed,
    DecreaseLaneSpeed,
    ResetLaneSpeed,
    IncreaseMasterVolume,
    DecreaseMasterVolume,
    IncreaseBgmVolume,
    DecreaseBgmVolume,
    IncreaseDrumVolume,
    DecreaseDrumVolume,
    IncreaseSongRate,
    DecreaseSongRate,
    ResetSongRate,
    ToggleMetronomeSound,
    ToggleDebugHud,
    CycleFpsCap,
}

pub const SYSTEM_ACTION_SETTINGS_ORDER: [SystemAction; 25] = [
    SystemAction::ToggleSettings,
    SystemAction::PauseToggle,
    SystemAction::RestartChart,
    SystemAction::SeekForward,
    SystemAction::SeekBackward,
    SystemAction::SeekToPreviousMeasure,
    SystemAction::SeekToNextMeasure,
    SystemAction::DecreaseTimingOffset,
    SystemAction::IncreaseTimingOffset,
    SystemAction::ResetTimingOffset,
    SystemAction::DecreaseLaneSpeed,
    SystemAction::IncreaseLaneSpeed,
    SystemAction::ResetLaneSpeed,
    SystemAction::DecreaseMasterVolume,
    SystemAction::IncreaseMasterVolume,
    SystemAction::DecreaseBgmVolume,
    SystemAction::IncreaseBgmVolume,
    SystemAction::DecreaseDrumVolume,
    SystemAction::IncreaseDrumVolume,
    SystemAction::DecreaseSongRate,
    SystemAction::ResetSongRate,
    SystemAction::IncreaseSongRate,
    SystemAction::ToggleMetronomeSound,
    SystemAction::ToggleDebugHud,
    SystemAction::CycleFpsCap,
];

impl SystemAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::ToggleSettings => "Toggle settings",
            Self::PauseToggle => "Pause / resume",
            Self::RestartChart => "Restart chart",
            Self::SeekForward => "Seek forward",
            Self::SeekBackward => "Seek backward",
            Self::SeekToPreviousMeasure => "Seek to start",
            Self::SeekToNextMeasure => "Seek to end",
            Self::IncreaseTimingOffset => "Timing offset +",
            Self::DecreaseTimingOffset => "Timing offset -",
            Self::ResetTimingOffset => "Reset timing offset",
            Self::IncreaseLaneSpeed => "Lane speed +",
            Self::DecreaseLaneSpeed => "Lane speed -",
            Self::ResetLaneSpeed => "Reset lane speed",
            Self::IncreaseMasterVolume => "Master volume +",
            Self::DecreaseMasterVolume => "Master volume -",
            Self::IncreaseBgmVolume => "BGM volume +",
            Self::DecreaseBgmVolume => "BGM volume -",
            Self::IncreaseDrumVolume => "Drum volume +",
            Self::DecreaseDrumVolume => "Drum volume -",
            Self::IncreaseSongRate => "Song rate +",
            Self::DecreaseSongRate => "Song rate -",
            Self::ResetSongRate => "Reset song rate",
            Self::ToggleMetronomeSound => "Toggle metronome",
            Self::ToggleDebugHud => "Toggle debug HUD",
            Self::CycleFpsCap => "Cycle FPS cap",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::ToggleSettings => {
                "Open or close the settings overlay (while paused in gameplay)."
            }
            Self::PauseToggle => "Pause or resume gameplay.",
            Self::RestartChart => "Restart chart. Double-tap or hold the bound key (default `).",
            Self::SeekForward => {
                "Seek playback forward five seconds. Practice mode only during play."
            }
            Self::SeekBackward => {
                "Seek playback backward five seconds. Practice mode only during play."
            }
            Self::SeekToPreviousMeasure => {
                "Seek to the start of the chart. Practice mode only during play."
            }
            Self::SeekToNextMeasure => {
                "Seek to the end of the chart. Practice mode only during play."
            }
            Self::IncreaseTimingOffset | Self::DecreaseTimingOffset | Self::ResetTimingOffset => {
                "Adjust persisted timing offset during play."
            }
            Self::IncreaseLaneSpeed | Self::DecreaseLaneSpeed | Self::ResetLaneSpeed => {
                "Adjust persisted lane scroll speed during play."
            }
            Self::IncreaseMasterVolume | Self::DecreaseMasterVolume => {
                "Adjust master output volume during play."
            }
            Self::IncreaseBgmVolume | Self::DecreaseBgmVolume => "Adjust BGM volume during play.",
            Self::IncreaseDrumVolume | Self::DecreaseDrumVolume => {
                "Adjust drum hit volume during play."
            }
            Self::IncreaseSongRate | Self::DecreaseSongRate | Self::ResetSongRate => {
                "Adjust song playback rate during play. Practice mode only."
            }
            Self::ToggleMetronomeSound => "Toggle metronome clicks during play.",
            Self::ToggleDebugHud => "Toggle the gameplay debug HUD.",
            Self::CycleFpsCap => "Cycle frame rate cap (VSync / 60 / 120 / 144 / 240 / Unlimited).",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MidiDeviceFilter {
    #[default]
    Any,
    Name(String),
}

impl MidiDeviceFilter {
    pub fn matches(&self, device_name: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Name(name) => name == device_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputSourceConfig {
    Keyboard {
        key: String,
    },
    MidiNote {
        #[serde(default)]
        device: MidiDeviceFilter,
        note: u8,
        channel: Option<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingTarget {
    DrumLane(DrumLane),
    GuitarLane(GuitarLane),
    BassLane(BassLane),
    System(SystemAction),
}

impl BindingTarget {
    /// Instrument this binding target belongs to. Used to route bindings
    /// to the right per-instrument resource at load time.
    pub fn instrument(&self) -> BindingInstrument {
        match self {
            Self::DrumLane(_) => BindingInstrument::Drums,
            Self::GuitarLane(_) => BindingInstrument::Guitar,
            Self::BassLane(_) => BindingInstrument::Bass,
            Self::System(_) => BindingInstrument::Drums,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBindingConfig {
    /// Defaults to `Drums` for legacy configs that pre-date the per-instrument
    /// refactor. Set automatically by the rebind UI when the user picks a
    /// guitar or bass lane target.
    #[serde(default)]
    pub instrument: BindingInstrument,
    pub source: InputSourceConfig,
    pub target: BindingTarget,
}

impl InputBindingConfig {
    /// Construct a binding with the instrument derived from the target.
    /// Lane targets (drum/guitar/bass) get their matching instrument;
    /// `System` targets fall back to `Drums` (legacy default).
    pub fn new(source: InputSourceConfig, target: BindingTarget) -> Self {
        Self {
            instrument: target.instrument(),
            source,
            target,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InputSource {
    Keyboard(KeyCode),
    MidiNote {
        device: MidiDeviceFilter,
        note: u8,
        channel: Option<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputBinding {
    pub(crate) source: InputSource,
    pub(crate) target: BindingTarget,
}
