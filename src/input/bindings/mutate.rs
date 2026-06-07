use bevy::prelude::*;

use super::defaults::default_system_bindings;
use super::keycodes::{keycode_from_name, keycode_name};
use super::types::{BindingTarget, DrumLane, InputBindingConfig, InputSourceConfig, SystemAction};

pub fn add_keyboard_lane_binding(
    bindings: &mut Vec<InputBindingConfig>,
    lane: usize,
    key: KeyCode,
) -> Result<(), &'static str> {
    let Some(name) = keycode_name(key) else {
        return Err("unsupported key");
    };
    let Some(target_lane) = DrumLane::from_index(lane) else {
        return Err("invalid lane");
    };

    bindings.retain(|binding| {
        !matches!(
            binding,
            InputBindingConfig {
                source: InputSourceConfig::Keyboard { key: existing },
                target: BindingTarget::DrumLane(_),
            } if existing == name
        )
    });

    let already_present = bindings.iter().any(|binding| {
        matches!(
            binding,
            InputBindingConfig {
                source: InputSourceConfig::Keyboard { key: existing },
                target: BindingTarget::DrumLane(found),
            } if existing == name && *found == target_lane
        )
    });

    if !already_present {
        bindings.push(InputBindingConfig {
            source: InputSourceConfig::Keyboard {
                key: name.to_string(),
            },
            target: BindingTarget::DrumLane(target_lane),
        });
    }

    Ok(())
}

pub fn add_midi_lane_binding(
    bindings: &mut Vec<InputBindingConfig>,
    lane: usize,
    device_name: &str,
    channel: u8,
    note: u8,
) -> Result<(), &'static str> {
    let Some(target_lane) = DrumLane::from_index(lane) else {
        return Err("invalid lane");
    };
    let source = InputSourceConfig::MidiNote {
        device: super::types::MidiDeviceFilter::Name(device_name.to_string()),
        note,
        channel: Some(channel),
    };

    bindings.retain(|binding| binding.source != source);

    let already_present = bindings.iter().any(|binding| {
        binding.source == source && binding.target == BindingTarget::DrumLane(target_lane)
    });
    if !already_present {
        bindings.push(InputBindingConfig {
            source,
            target: BindingTarget::DrumLane(target_lane),
        });
    }

    Ok(())
}

pub fn keyboard_key_for_action(
    bindings: &[InputBindingConfig],
    action: SystemAction,
) -> Option<KeyCode> {
    bindings.iter().find_map(|binding| match binding {
        InputBindingConfig {
            source: InputSourceConfig::Keyboard { key },
            target: BindingTarget::System(found),
        } if *found == action => keycode_from_name(key),
        _ => None,
    })
}

pub fn set_system_keyboard_binding(
    bindings: &mut Vec<InputBindingConfig>,
    action: SystemAction,
    key: KeyCode,
) -> Result<(), &'static str> {
    let Some(name) = keycode_name(key) else {
        return Err("unsupported key");
    };

    bindings.retain(|binding| {
        !matches!(
            binding,
            InputBindingConfig {
                source: InputSourceConfig::Keyboard { key: existing },
                ..
            } if existing == name
        )
    });
    bindings.retain(|binding| {
        !matches!(
            binding,
            InputBindingConfig {
                source: InputSourceConfig::Keyboard { .. },
                target: BindingTarget::System(existing),
            } if *existing == action
        )
    });
    bindings.push(InputBindingConfig {
        source: InputSourceConfig::Keyboard {
            key: name.to_string(),
        },
        target: BindingTarget::System(action),
    });
    Ok(())
}

pub fn reset_system_keyboard_binding(bindings: &mut Vec<InputBindingConfig>, action: SystemAction) {
    bindings.retain(|binding| {
        !matches!(
            binding,
            InputBindingConfig {
                source: InputSourceConfig::Keyboard { .. },
                target: BindingTarget::System(existing),
            } if *existing == action
        )
    });

    for default in default_system_bindings() {
        if matches!(default.target, BindingTarget::System(found) if found == action) {
            bindings.push(default);
        }
    }
}

pub fn lane_binding_indices(bindings: &[InputBindingConfig], lane: usize) -> Vec<usize> {
    let Some(target_lane) = DrumLane::from_index(lane) else {
        return Vec::new();
    };

    bindings
        .iter()
        .enumerate()
        .filter_map(|(index, binding)| {
            matches!(
                binding.target,
                BindingTarget::DrumLane(found) if found == target_lane
            )
            .then_some(index)
        })
        .collect()
}

pub fn remove_lane_binding_at(
    bindings: &mut Vec<InputBindingConfig>,
    lane: usize,
    entry_index: usize,
) -> bool {
    let indices = lane_binding_indices(bindings, lane);
    let Some(&binding_index) = indices.get(entry_index) else {
        return false;
    };
    bindings.remove(binding_index);
    true
}
