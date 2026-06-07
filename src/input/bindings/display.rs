use bevy::prelude::*;

use super::keycodes::{keycode_display_name, keycode_from_name};
use super::mutate::keyboard_key_for_action;
use super::types::{
    BindingTarget, DrumLane, InputBindingConfig, InputSourceConfig, MidiDeviceFilter, SystemAction,
};

pub fn system_action_binding_value(
    bindings: &[InputBindingConfig],
    action: SystemAction,
) -> String {
    keyboard_key_for_action(bindings, action)
        .map(|key| keycode_display_name(key).to_string())
        .unwrap_or_else(|| "—".to_string())
}

pub fn lane_bindings_value(
    bindings: &[InputBindingConfig],
    lane: usize,
    cursor: Option<usize>,
) -> String {
    let indices = super::mutate::lane_binding_indices(bindings, lane);
    if indices.is_empty() {
        return "—".to_string();
    }

    let mut keyboard = Vec::new();
    let mut midi = Vec::new();
    for (entry_index, binding_index) in indices.into_iter().enumerate() {
        let Some(binding) = bindings.get(binding_index) else {
            continue;
        };
        let label = match &binding.source {
            InputSourceConfig::Keyboard { key } => keycode_from_name(key)
                .map(|code| keycode_display_name(code).to_string())
                .unwrap_or_else(|| key.clone()),
            InputSourceConfig::MidiNote {
                device,
                note,
                channel,
            } => midi_binding_label(device, *note, *channel),
        };
        let highlighted = cursor == Some(entry_index);
        let rendered = if highlighted {
            format!("[{label}]")
        } else {
            label
        };
        match &binding.source {
            InputSourceConfig::Keyboard { .. } => keyboard.push(rendered),
            InputSourceConfig::MidiNote { .. } => midi.push(rendered),
        }
    }

    let mut parts = Vec::new();
    if !keyboard.is_empty() {
        parts.push(keyboard.join(", "));
    }
    if !midi.is_empty() {
        parts.push(format!("MIDI: {}", midi.join(", ")));
    }
    parts.join(" | ")
}

fn midi_binding_label(device: &MidiDeviceFilter, note: u8, channel: Option<u8>) -> String {
    let mut label = midi_note_display_name(note);
    if let Some(channel) = channel {
        label.push_str(&format!(" ch{channel}"));
    }
    if let MidiDeviceFilter::Name(name) = device {
        let short = name
            .split_whitespace()
            .next()
            .filter(|part| !part.is_empty())
            .unwrap_or(name.as_str());
        label.push_str(&format!(" @{short}"));
    }
    label
}

fn midi_note_display_name(note: u8) -> String {
    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note as i32 / 12) - 1;
    let name = NOTE_NAMES[(note % 12) as usize];
    format!("{name}{octave}")
}

pub fn keyboard_summary_for_lane(bindings: &[InputBindingConfig], lane: usize) -> String {
    let keys = keyboard_keys_for_lane_config(bindings, lane);
    let midi_count = midi_note_bindings_for_lane(bindings, lane).len();

    let keyboard = match keys.as_slice() {
        [] => "—".to_string(),
        [only] => keycode_display_name(*only).to_string(),
        [first, second] => format!(
            "{}/{}",
            keycode_display_name(*first),
            keycode_display_name(*second)
        ),
        [first, rest @ ..] => format!("{} +{}", keycode_display_name(*first), rest.len()),
    };

    if midi_count == 0 {
        keyboard
    } else {
        format!("{keyboard} | MIDI×{midi_count}")
    }
}

pub fn keyboard_keys_for_lane_config(bindings: &[InputBindingConfig], lane: usize) -> Vec<KeyCode> {
    let Some(target_lane) = DrumLane::from_index(lane) else {
        return Vec::new();
    };

    bindings
        .iter()
        .filter_map(|binding| match binding {
            InputBindingConfig {
                source: InputSourceConfig::Keyboard { key },
                target: BindingTarget::DrumLane(found),
            } if *found == target_lane => keycode_from_name(key),
            _ => None,
        })
        .collect()
}

pub fn midi_note_bindings_for_lane(
    bindings: &[InputBindingConfig],
    lane: usize,
) -> Vec<(MidiDeviceFilter, u8, Option<u8>)> {
    let Some(target_lane) = DrumLane::from_index(lane) else {
        return Vec::new();
    };

    bindings
        .iter()
        .filter_map(|binding| match binding {
            InputBindingConfig {
                source:
                    InputSourceConfig::MidiNote {
                        device,
                        note,
                        channel,
                    },
                target: BindingTarget::DrumLane(found),
            } if *found == target_lane => Some((device.clone(), *note, *channel)),
            _ => None,
        })
        .collect()
}
