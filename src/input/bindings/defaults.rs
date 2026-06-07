use bevy::prelude::*;

use super::keycodes::keycode_from_name;
use super::types::{
    BindingTarget, DEFAULT_LANE_KEY_NAMES, DrumLane, InputBindingConfig, InputSourceConfig,
    LANE_COUNT, MidiDeviceFilter, SystemAction,
};
use crate::input::lanes::{
    LANE_BD, LANE_CY, LANE_FT, LANE_HH, LANE_HT, LANE_LC, LANE_LP, LANE_LT, LANE_RD, LANE_SD,
};

pub fn default_lane_key_names() -> [String; LANE_COUNT] {
    std::array::from_fn(|lane| DEFAULT_LANE_KEY_NAMES[lane].to_string())
}

pub fn default_keycode(lane: usize) -> KeyCode {
    keycode_from_name(DEFAULT_LANE_KEY_NAMES[lane]).expect("valid default lane key")
}

pub fn default_input_bindings() -> Vec<InputBindingConfig> {
    let mut bindings = Vec::new();

    for (lane, name) in DEFAULT_LANE_KEY_NAMES.iter().enumerate() {
        bindings.push(InputBindingConfig {
            source: InputSourceConfig::Keyboard {
                key: name.to_string(),
            },
            target: BindingTarget::DrumLane(DrumLane::from_index(lane).expect("valid lane")),
        });
    }

    for (note, lane) in default_midi_lane_notes() {
        bindings.push(InputBindingConfig {
            source: InputSourceConfig::MidiNote {
                device: MidiDeviceFilter::Any,
                note,
                channel: None,
            },
            target: BindingTarget::DrumLane(DrumLane::from_index(lane).expect("valid lane")),
        });
    }

    bindings.extend(default_system_bindings());
    bindings
}

pub fn default_input_bindings_with_lane_keys(
    lane_keys: &[String; LANE_COUNT],
) -> Vec<InputBindingConfig> {
    let mut bindings = default_input_bindings();
    bindings.retain(|binding| {
        !matches!(
            binding,
            InputBindingConfig {
                source: InputSourceConfig::Keyboard { .. },
                target: BindingTarget::DrumLane(_),
            }
        )
    });

    for (lane, key) in lane_keys.iter().enumerate() {
        bindings.push(InputBindingConfig {
            source: InputSourceConfig::Keyboard { key: key.clone() },
            target: BindingTarget::DrumLane(DrumLane::from_index(lane).expect("valid lane")),
        });
    }

    bindings
}

pub(crate) fn default_system_bindings() -> Vec<InputBindingConfig> {
    use SystemAction::*;

    [
        ("F1", ToggleSettings),
        ("Escape", PauseToggle),
        ("Backquote", RestartChart),
        ("PageUp", SeekForward),
        ("PageDown", SeekBackward),
        ("Home", SeekToPreviousMeasure),
        ("End", SeekToNextMeasure),
        ("BracketLeft", DecreaseTimingOffset),
        ("BracketRight", IncreaseTimingOffset),
        ("Backslash", ResetTimingOffset),
        ("Minus", DecreaseLaneSpeed),
        ("Equal", IncreaseLaneSpeed),
        ("Digit0", ResetLaneSpeed),
        ("KeyZ", DecreaseMasterVolume),
        ("KeyX", IncreaseMasterVolume),
        ("KeyC", DecreaseBgmVolume),
        ("KeyB", IncreaseBgmVolume),
        ("KeyN", DecreaseDrumVolume),
        ("Comma", IncreaseDrumVolume),
        ("Digit9", DecreaseSongRate),
        ("Digit8", ResetSongRate),
        ("Digit7", IncreaseSongRate),
        ("KeyM", ToggleMetronomeSound),
        ("KeyV", ToggleDebugHud),
        ("F6", ToggleVsync),
        ("F3", DecreaseLaneSpeed),
        ("F4", IncreaseLaneSpeed),
    ]
    .into_iter()
    .map(|(key, action)| InputBindingConfig {
        source: InputSourceConfig::Keyboard {
            key: key.to_string(),
        },
        target: BindingTarget::System(action),
    })
    .collect()
}

fn default_midi_lane_notes() -> impl IntoIterator<Item = (u8, usize)> {
    [
        (36, LANE_BD),
        (38, LANE_SD),
        (41, LANE_FT),
        (42, LANE_HH),
        (44, LANE_HH),
        (46, LANE_HH),
        (45, LANE_LT),
        (48, LANE_HT),
        (49, LANE_CY),
        (51, LANE_RD),
        (55, LANE_LP),
        (57, LANE_LC),
        (60, LANE_BD),
        (62, LANE_SD),
        (64, LANE_FT),
        (65, LANE_HH),
        (67, LANE_LP),
        (69, LANE_LT),
        (71, LANE_HT),
        (72, LANE_CY),
        (74, LANE_RD),
        (76, LANE_LC),
    ]
}
