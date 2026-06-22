use bevy::prelude::*;

use super::{
    BassLane, BindingInstrument, BindingTarget, DrumLane, GuitarLane, InputBindingConfig,
    InputBindings, InputSourceConfig, MidiDeviceFilter, SystemAction, add_keyboard_binding,
    add_keyboard_lane_binding, add_midi_binding, add_midi_lane_binding, default_input_bindings,
    keyboard_key_for_action, keyboard_keys_for_lane_config, keyboard_keys_for_target_config,
    keyboard_summary_for_target, lane_binding_indices, remove_lane_binding_at,
    remove_target_binding_at, set_system_keyboard_binding, target_binding_indices,
    target_bindings_value,
};
use crate::input::lanes;
use crate::input::lanes::LANE_BD;

#[test]
fn default_bindings_include_keyboard_and_midi() {
    let bindings = default_input_bindings();
    assert!(bindings.iter().any(|binding| matches!(
        binding,
        InputBindingConfig {
            instrument: _,
            source: InputSourceConfig::Keyboard { key },
            target: BindingTarget::DrumLane(DrumLane::Bd),
        } if key == "KeyA"
    )));
    assert!(bindings.iter().any(|binding| matches!(
        binding,
        InputBindingConfig {
            instrument: _,
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
                instrument: _,
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
            instrument: _,
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

#[test]
fn binding_instrument_default_is_drums() {
    assert_eq!(BindingInstrument::default(), BindingInstrument::Drums);
    assert_eq!(BindingInstrument::ALL.len(), 3);
}

#[test]
fn binding_target_instrument_matches_variant() {
    assert_eq!(
        BindingTarget::DrumLane(DrumLane::Bd).instrument(),
        BindingInstrument::Drums
    );
    assert_eq!(
        BindingTarget::GuitarLane(GuitarLane::R).instrument(),
        BindingInstrument::Guitar
    );
    assert_eq!(
        BindingTarget::BassLane(BassLane::R).instrument(),
        BindingInstrument::Bass
    );
    assert_eq!(
        BindingTarget::System(SystemAction::PauseToggle).instrument(),
        BindingInstrument::Drums
    );
}

#[test]
fn guitar_and_bass_lane_visible_sets_have_expected_sizes() {
    assert_eq!(GuitarLane::VISIBLE_LANES.len(), 5);
    assert_eq!(BassLane::VISIBLE_LANES.len(), 4);
    // R/G/B must be present in both.
    assert!(GuitarLane::VISIBLE_LANES.contains(&GuitarLane::R));
    assert!(GuitarLane::VISIBLE_LANES.contains(&GuitarLane::G));
    assert!(GuitarLane::VISIBLE_LANES.contains(&GuitarLane::B));
    assert!(BassLane::VISIBLE_LANES.contains(&BassLane::R));
    assert!(BassLane::VISIBLE_LANES.contains(&BassLane::G));
    assert!(BassLane::VISIBLE_LANES.contains(&BassLane::B));
}

#[test]
fn input_binding_config_new_derives_instrument_from_target() {
    let binding = InputBindingConfig::new(
        InputSourceConfig::Keyboard { key: "KeyA".into() },
        BindingTarget::GuitarLane(GuitarLane::R),
    );
    assert_eq!(binding.instrument, BindingInstrument::Guitar);
    assert_eq!(binding.target.instrument(), BindingInstrument::Guitar);
}

#[test]
fn legacy_v13_config_round_trips_with_default_instrument() {
    // v13 RON did not have the `instrument` field on each binding. With
    // `#[serde(default)]` on the field, deserialization fills `Drums`.
    let v13_text = r#"(
        source: Keyboard(key: "KeyA"),
        target: DrumLane(Bd),
    )"#;
    let parsed: InputBindingConfig = ron::de::from_str(v13_text).expect("parse v13 binding");
    assert_eq!(parsed.instrument, BindingInstrument::Drums);
    // Round-trip preserves the default instrument.
    let serialized = ron::ser::to_string(&parsed).expect("serialize");
    let reparsed: InputBindingConfig = ron::de::from_str(&serialized).expect("reparse");
    assert_eq!(reparsed, parsed);
}

#[test]
fn input_bindings_for_instrument_filters_correctly() {
    let bindings = InputBindings::from_config(&[
        InputBindingConfig::new(
            InputSourceConfig::Keyboard { key: "KeyA".into() },
            BindingTarget::DrumLane(DrumLane::Bd),
        ),
        InputBindingConfig::new(
            InputSourceConfig::Keyboard { key: "KeyQ".into() },
            BindingTarget::GuitarLane(GuitarLane::R),
        ),
        InputBindingConfig::new(
            InputSourceConfig::Keyboard { key: "KeyW".into() },
            BindingTarget::BassLane(BassLane::R),
        ),
    ]);
    let drum_count = bindings.for_instrument(BindingInstrument::Drums).count();
    let guitar_count = bindings.for_instrument(BindingInstrument::Guitar).count();
    let bass_count = bindings.for_instrument(BindingInstrument::Bass).count();
    assert_eq!(drum_count, 1);
    assert_eq!(guitar_count, 1);
    assert_eq!(bass_count, 1);
}

#[test]
fn add_keyboard_binding_scopes_per_instrument() {
    // The same KeyCode can be bound to a drum and a guitar target
    // simultaneously; conflict resolution only fires within the same
    // instrument profile.
    let mut bindings: Vec<InputBindingConfig> = Vec::new();
    add_keyboard_binding(
        &mut bindings,
        BindingTarget::DrumLane(DrumLane::Bd),
        KeyCode::KeyA,
    )
    .expect("drum KeyA add");
    add_keyboard_binding(
        &mut bindings,
        BindingTarget::GuitarLane(GuitarLane::R),
        KeyCode::KeyA,
    )
    .expect("guitar KeyA add");
    // Two bindings, distinct instruments, same source.
    assert_eq!(bindings.len(), 2);
    let drum_count = target_binding_indices(&bindings, BindingTarget::DrumLane(DrumLane::Bd)).len();
    let guitar_count =
        target_binding_indices(&bindings, BindingTarget::GuitarLane(GuitarLane::R)).len();
    assert_eq!(drum_count, 1);
    assert_eq!(guitar_count, 1);
    // The display summary distinguishes instruments.
    assert_eq!(
        keyboard_summary_for_target(&bindings, BindingTarget::DrumLane(DrumLane::Bd)),
        "A"
    );
    assert_eq!(
        keyboard_summary_for_target(&bindings, BindingTarget::GuitarLane(GuitarLane::R)),
        "A"
    );
}

#[test]
fn add_keyboard_binding_replaces_on_same_instrument_and_reports_conflict() {
    let mut bindings: Vec<InputBindingConfig> = Vec::new();
    // First binding: guitar R = KeyA.
    let result = add_keyboard_binding(
        &mut bindings,
        BindingTarget::GuitarLane(GuitarLane::R),
        KeyCode::KeyA,
    )
    .expect("first");
    assert_eq!(result.replaced, None);
    // Re-binding guitar G to the same key should report the previous
    // target so the UI can show a conflict message, and the new
    // binding should take the slot.
    let result = add_keyboard_binding(
        &mut bindings,
        BindingTarget::GuitarLane(GuitarLane::G),
        KeyCode::KeyA,
    )
    .expect("second");
    assert_eq!(
        result.replaced,
        Some(BindingTarget::GuitarLane(GuitarLane::R))
    );
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].target, BindingTarget::GuitarLane(GuitarLane::G));
    // Keys for R: empty. Keys for G: [A].
    let r_keys =
        keyboard_keys_for_target_config(&bindings, BindingTarget::GuitarLane(GuitarLane::R));
    let g_keys =
        keyboard_keys_for_target_config(&bindings, BindingTarget::GuitarLane(GuitarLane::G));
    assert!(r_keys.is_empty());
    assert_eq!(g_keys, vec![KeyCode::KeyA]);
}

#[test]
fn add_midi_binding_replaces_per_instrument() {
    let mut bindings: Vec<InputBindingConfig> = Vec::new();
    add_midi_binding(
        &mut bindings,
        BindingTarget::BassLane(BassLane::R),
        "TD-17",
        9,
        36,
    )
    .expect("bass add");
    // Same device/channel/note on guitar: separate binding, no conflict.
    add_midi_binding(
        &mut bindings,
        BindingTarget::GuitarLane(GuitarLane::R),
        "TD-17",
        9,
        36,
    )
    .expect("guitar add");
    assert_eq!(bindings.len(), 2);
    // Re-assign the same note to a different bass target; the original
    // bass binding is replaced.
    let result = add_midi_binding(
        &mut bindings,
        BindingTarget::BassLane(BassLane::G),
        "TD-17",
        9,
        36,
    )
    .expect("bass re-add");
    assert_eq!(result.replaced, Some(BindingTarget::BassLane(BassLane::R)));
    assert_eq!(bindings.len(), 2); // bass entry replaced, guitar intact
    let bass_g = target_binding_indices(&bindings, BindingTarget::BassLane(BassLane::G));
    let bass_r = target_binding_indices(&bindings, BindingTarget::BassLane(BassLane::R));
    assert_eq!(bass_g.len(), 1);
    assert_eq!(bass_r.len(), 0);
}

#[test]
fn remove_target_binding_at_drops_the_target_only() {
    let mut bindings: Vec<InputBindingConfig> = Vec::new();
    add_keyboard_binding(
        &mut bindings,
        BindingTarget::GuitarLane(GuitarLane::R),
        KeyCode::KeyA,
    )
    .unwrap();
    add_keyboard_binding(
        &mut bindings,
        BindingTarget::GuitarLane(GuitarLane::G),
        KeyCode::KeyS,
    )
    .unwrap();
    assert_eq!(bindings.len(), 2);
    let removed =
        remove_target_binding_at(&mut bindings, BindingTarget::GuitarLane(GuitarLane::R), 0);
    assert!(removed);
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].target, BindingTarget::GuitarLane(GuitarLane::G));
}

#[test]
fn target_bindings_value_formats_keyboard_and_midi() {
    let mut bindings: Vec<InputBindingConfig> = Vec::new();
    add_keyboard_binding(
        &mut bindings,
        BindingTarget::DrumLane(DrumLane::Sd),
        KeyCode::KeyS,
    )
    .unwrap();
    add_midi_binding(
        &mut bindings,
        BindingTarget::DrumLane(DrumLane::Sd),
        "TD-17",
        10,
        38,
    )
    .unwrap();
    let s = target_bindings_value(&bindings, BindingTarget::DrumLane(DrumLane::Sd), None);
    // Both keyboard and MIDI should appear, joined with " | ".
    assert!(s.contains("S"));
    assert!(s.contains("MIDI"));
}
