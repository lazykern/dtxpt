use bevy::prelude::*;

use super::{
    BindingTarget, DrumLane, InputBindingConfig, InputSourceConfig, MidiDeviceFilter, SystemAction,
    add_keyboard_lane_binding, add_midi_lane_binding, default_input_bindings,
    keyboard_key_for_action, keyboard_keys_for_lane_config, lane_binding_indices,
    remove_lane_binding_at, set_system_keyboard_binding,
};
use crate::input::lanes;
use crate::input::lanes::LANE_BD;

#[test]
fn default_bindings_include_keyboard_and_midi() {
    let bindings = default_input_bindings();
    assert!(bindings.iter().any(|binding| matches!(
        binding,
        InputBindingConfig {
            source: InputSourceConfig::Keyboard { key },
            target: BindingTarget::DrumLane(DrumLane::Bd),
        } if key == "KeyA"
    )));
    assert!(bindings.iter().any(|binding| matches!(
        binding,
        InputBindingConfig {
            source: InputSourceConfig::MidiNote { note: 36, .. },
            target: BindingTarget::DrumLane(DrumLane::Bd),
        }
    )));
}

#[test]
fn adding_keyboard_binding_moves_conflict() {
    let mut bindings = default_input_bindings();
    add_keyboard_lane_binding(&mut bindings, LANE_BD, KeyCode::KeyS).unwrap();

    let bd = keyboard_keys_for_lane_config(&bindings, LANE_BD);
    let sd = keyboard_keys_for_lane_config(&bindings, lanes::LANE_SD);

    assert!(bd.contains(&KeyCode::KeyS));
    assert!(!sd.contains(&KeyCode::KeyS));
}

#[test]
fn add_midi_lane_binding_is_device_specific() {
    let mut bindings = default_input_bindings();
    add_midi_lane_binding(&mut bindings, LANE_BD, "Roland TD-17", 9, 36).unwrap();

    let has_specific = bindings.iter().any(|binding| {
        matches!(
            binding,
            InputBindingConfig {
                source: InputSourceConfig::MidiNote {
                    device: MidiDeviceFilter::Name(name),
                    note: 36,
                    channel: Some(9),
                },
                target: BindingTarget::DrumLane(DrumLane::Bd),
            } if name == "Roland TD-17"
        )
    });
    assert!(has_specific);
}

#[test]
fn default_bindings_use_escape_pause_and_backquote_restart() {
    let bindings = default_input_bindings();
    assert_eq!(
        keyboard_key_for_action(&bindings, SystemAction::PauseToggle),
        Some(KeyCode::Escape)
    );
    assert_eq!(
        keyboard_key_for_action(&bindings, SystemAction::RestartChart),
        Some(KeyCode::Backquote)
    );
    assert!(bindings.iter().all(|binding| !matches!(
        binding,
        InputBindingConfig {
            source: InputSourceConfig::Keyboard { key },
            target: BindingTarget::System(SystemAction::PauseToggle),
        } if key == "Space"
    )));
}

#[test]
fn set_system_keyboard_binding_replaces_action_key() {
    let mut bindings = default_input_bindings();
    set_system_keyboard_binding(&mut bindings, SystemAction::PauseToggle, KeyCode::KeyP).unwrap();

    assert_eq!(
        keyboard_key_for_action(&bindings, SystemAction::PauseToggle),
        Some(KeyCode::KeyP)
    );
    assert_eq!(
        keyboard_key_for_action(&bindings, SystemAction::RestartChart),
        Some(KeyCode::Backquote)
    );
}

#[test]
fn remove_lane_binding_keeps_other_lanes() {
    let mut bindings = default_input_bindings();
    add_keyboard_lane_binding(&mut bindings, LANE_BD, KeyCode::KeyG).unwrap();
    let before = lane_binding_indices(&bindings, LANE_BD).len();
    assert!(before >= 2);

    assert!(remove_lane_binding_at(&mut bindings, LANE_BD, 0));
    let keys = keyboard_keys_for_lane_config(&bindings, LANE_BD);
    assert!(!keys.contains(&KeyCode::KeyA));
    assert!(keyboard_keys_for_lane_config(&bindings, lanes::LANE_SD).contains(&KeyCode::KeyS));
}
