use bevy::prelude::*;

use super::defaults::default_system_bindings;
use super::keycodes::{keycode_from_name, keycode_name};
use super::types::{
    BindingInstrument, BindingTarget, DrumLane, InputBindingConfig, InputSourceConfig, SystemAction,
};

/// Result of adding a binding. Reports whether an existing binding for
/// the same source was replaced (and which target it had), so the
/// rebind UI can show a conflict message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddBindingResult {
    pub replaced: Option<BindingTarget>,
}

/// Add a keyboard binding for `target` on the matching instrument.
/// Replaces any existing keyboard binding for the same key on that
/// instrument (mirrors BocuD's silent override). `target`'s
/// instrument must agree with the binding's intended instrument; we
/// derive it from the target to keep callers terse.
pub fn add_keyboard_binding(
    bindings: &mut Vec<InputBindingConfig>,
    target: BindingTarget,
    key: KeyCode,
) -> Result<AddBindingResult, &'static str> {
    let Some(name) = keycode_name(key) else {
        return Err("unsupported key");
    };
    let instrument = target.instrument();
    let source = InputSourceConfig::Keyboard {
        key: name.to_string(),
    };

    let replaced = bindings
        .iter()
        .find(|binding| {
            binding.instrument == instrument && binding.source == source && binding.target != target
        })
        .map(|binding| binding.target.clone());

    bindings.retain(|binding| !(binding.instrument == instrument && binding.source == source));

    let already_present = bindings
        .iter()
        .any(|binding| binding.instrument == instrument && binding.source == source);

    if !already_present {
        bindings.push(InputBindingConfig {
            instrument,
            source,
            target,
        });
    }

    Ok(AddBindingResult { replaced })
}

/// Add a MIDI-note binding for `target` on the matching instrument.
/// Replaces any existing binding for the same source on that instrument.
pub fn add_midi_binding(
    bindings: &mut Vec<InputBindingConfig>,
    target: BindingTarget,
    device_name: &str,
    channel: u8,
    note: u8,
) -> Result<AddBindingResult, &'static str> {
    let instrument = target.instrument();
    let source = InputSourceConfig::MidiNote {
        device: super::types::MidiDeviceFilter::Name(device_name.to_string()),
        note,
        channel: Some(channel),
    };

    let replaced = bindings
        .iter()
        .find(|binding| {
            binding.instrument == instrument && binding.source == source && binding.target != target
        })
        .map(|binding| binding.target.clone());

    bindings.retain(|binding| !(binding.instrument == instrument && binding.source == source));

    let already_present = bindings
        .iter()
        .any(|binding| binding.instrument == instrument && binding.source == source);

    if !already_present {
        bindings.push(InputBindingConfig {
            instrument,
            source,
            target,
        });
    }

    Ok(AddBindingResult { replaced })
}

/// Drum-only thin shim. `lane` is the index into [`LANES`](crate::input::LANES).
pub fn add_keyboard_lane_binding(
    bindings: &mut Vec<InputBindingConfig>,
    lane: usize,
    key: KeyCode,
) -> Result<(), &'static str> {
    let Some(target_lane) = DrumLane::from_index(lane) else {
        return Err("invalid lane");
    };
    add_keyboard_binding(bindings, BindingTarget::DrumLane(target_lane), key)?;
    Ok(())
}

/// Drum-only thin shim. `lane` is the index into [`LANES`](crate::input::LANES).
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
    add_midi_binding(
        bindings,
        BindingTarget::DrumLane(target_lane),
        device_name,
        channel,
        note,
    )?;
    Ok(())
}

pub fn keyboard_key_for_action(
    bindings: &[InputBindingConfig],
    action: SystemAction,
) -> Option<KeyCode> {
    bindings.iter().find_map(|binding| match binding {
        InputBindingConfig {
            instrument: BindingInstrument::Drums,
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
                instrument: BindingInstrument::Drums,
                source: InputSourceConfig::Keyboard { key: existing },
                ..
            } if existing == name
        )
    });
    bindings.retain(|binding| {
        !matches!(
            binding,
            InputBindingConfig {
                instrument: BindingInstrument::Drums,
                source: InputSourceConfig::Keyboard { .. },
                target: BindingTarget::System(existing),
            } if *existing == action
        )
    });
    bindings.push(InputBindingConfig {
        instrument: BindingInstrument::Drums,
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
                instrument: BindingInstrument::Drums,
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

pub fn target_binding_indices(
    bindings: &[InputBindingConfig],
    target: BindingTarget,
) -> Vec<usize> {
    let instrument = target.instrument();
    bindings
        .iter()
        .enumerate()
        .filter_map(|(index, binding)| {
            (binding.instrument == instrument && binding.target == target).then_some(index)
        })
        .collect()
}

pub fn lane_binding_indices(bindings: &[InputBindingConfig], lane: usize) -> Vec<usize> {
    let Some(target_lane) = DrumLane::from_index(lane) else {
        return Vec::new();
    };
    target_binding_indices(bindings, BindingTarget::DrumLane(target_lane))
}

pub fn remove_target_binding_at(
    bindings: &mut Vec<InputBindingConfig>,
    target: BindingTarget,
    entry_index: usize,
) -> bool {
    let indices = target_binding_indices(bindings, target);
    let Some(&binding_index) = indices.get(entry_index) else {
        return false;
    };
    bindings.remove(binding_index);
    true
}

pub fn remove_lane_binding_at(
    bindings: &mut Vec<InputBindingConfig>,
    lane: usize,
    entry_index: usize,
) -> bool {
    let Some(target_lane) = DrumLane::from_index(lane) else {
        return false;
    };
    remove_target_binding_at(bindings, BindingTarget::DrumLane(target_lane), entry_index)
}
