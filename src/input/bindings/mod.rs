mod defaults;
mod display;
mod keycodes;
mod mutate;
mod runtime;
mod types;

#[cfg(test)]
mod tests;

pub use defaults::{
    default_input_bindings, default_input_bindings_with_lane_keys, default_keycode,
    default_lane_key_names,
};
pub use display::{
    keyboard_keys_for_lane_config, keyboard_keys_for_target_config, keyboard_summary_for_lane,
    keyboard_summary_for_target, lane_bindings_value, midi_note_bindings_for_lane,
    midi_note_bindings_for_target, system_action_binding_value, target_bindings_value,
};
pub use keycodes::{keycode_display_name, keycode_from_name, keycode_name};
pub use mutate::{
    AddBindingResult, add_keyboard_binding, add_keyboard_lane_binding, add_midi_binding,
    add_midi_lane_binding, keyboard_key_for_action, lane_binding_indices, remove_lane_binding_at,
    remove_target_binding_at, reset_system_keyboard_binding, set_system_keyboard_binding,
    target_binding_indices,
};
pub use runtime::{InputBindings, LaneTriggerSource};
pub use types::{
    BassLane, BindingInstrument, BindingTarget, DEFAULT_LANE_KEY_NAMES, DrumLane, GuitarLane,
    InputBindingConfig, InputSourceConfig, LANE_COUNT, MidiDeviceFilter,
    SYSTEM_ACTION_SETTINGS_ORDER, SystemAction,
};
