use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::input::lanes::{
    LANE_BD, LANE_CY, LANE_FT, LANE_HH, LANE_HT, LANE_LC, LANE_LP, LANE_LT, LANE_RD, LANE_SD,
};

pub const LANE_COUNT: usize = 10;

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
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
            _ => return None,
        })
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
    ToggleVsync,
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
    SystemAction::ToggleVsync,
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
            Self::ToggleVsync => "Toggle VSync",
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
            Self::ToggleVsync => "Toggle vertical sync for the game window.",
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
    System(SystemAction),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBindingConfig {
    pub source: InputSourceConfig,
    pub target: BindingTarget,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlayMode {
    #[default]
    Normal,
    Practice,
    Auto,
}

impl PlayMode {
    pub fn label(self) -> &'static str {
        match self {
            PlayMode::Normal => "Normal",
            PlayMode::Practice => "Practice",
            PlayMode::Auto => "Auto",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            PlayMode::Normal => "Gauge can fail the stage on poor/miss judgements.",
            PlayMode::Practice => "No stage fail — practice charts without ending early.",
            PlayMode::Auto => "Autoplay — lane hits are simulated. Use as a test rig or to watch a chart.",
        }
    }

    pub fn next(self) -> Self {
        match self {
            PlayMode::Normal => PlayMode::Practice,
            PlayMode::Practice => PlayMode::Auto,
            PlayMode::Auto => PlayMode::Normal,
        }
    }

    /// Whether lane input is processed. False for Auto (autoplay drives hits).
    pub fn player_drives_lanes(self) -> bool {
        !matches!(self, PlayMode::Auto)
    }
}
