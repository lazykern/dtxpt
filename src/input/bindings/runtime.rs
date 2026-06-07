use bevy::prelude::*;

use super::keycodes::keycode_from_name;
use super::types::{
    BindingTarget, DrumLane, InputBinding, InputBindingConfig, InputSource, InputSourceConfig,
    SystemAction,
};
use crate::input::midi::MidiNoteEvent;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct InputBindings {
    pub(crate) bindings: Vec<InputBinding>,
}

impl InputBindings {
    pub fn from_config(entries: &[InputBindingConfig]) -> Self {
        let bindings = entries
            .iter()
            .filter_map(|entry| {
                let source = match &entry.source {
                    InputSourceConfig::Keyboard { key } => {
                        Some(InputSource::Keyboard(keycode_from_name(key)?))
                    }
                    InputSourceConfig::MidiNote {
                        device,
                        note,
                        channel,
                    } => Some(InputSource::MidiNote {
                        device: device.clone(),
                        note: *note,
                        channel: *channel,
                    }),
                }?;
                Some(InputBinding {
                    source,
                    target: entry.target.clone(),
                })
            })
            .collect();
        Self { bindings }
    }

    pub fn lane_triggered(
        &self,
        lane: usize,
        keyboard: &ButtonInput<KeyCode>,
        midi_events: &[MidiNoteEvent],
    ) -> bool {
        let Some(target_lane) = DrumLane::from_index(lane) else {
            return false;
        };
        self.bindings.iter().any(|binding| {
            binding.target == BindingTarget::DrumLane(target_lane)
                && binding_triggered(&binding.source, keyboard, midi_events)
        })
    }

    pub fn action_just_pressed(
        &self,
        action: SystemAction,
        keyboard: &ButtonInput<KeyCode>,
        midi_events: &[MidiNoteEvent],
    ) -> bool {
        self.bindings.iter().any(|binding| {
            binding.target == BindingTarget::System(action)
                && binding_triggered(&binding.source, keyboard, midi_events)
        })
    }

    pub fn keyboard_keys_for_action(&self, action: SystemAction) -> Vec<KeyCode> {
        self.bindings
            .iter()
            .filter_map(|binding| match (&binding.source, &binding.target) {
                (InputSource::Keyboard(key), BindingTarget::System(found)) if *found == action => {
                    Some(*key)
                }
                _ => None,
            })
            .collect()
    }

    pub fn keyboard_keys_for_lane(&self, lane: usize) -> Vec<KeyCode> {
        let Some(target_lane) = DrumLane::from_index(lane) else {
            return Vec::new();
        };
        self.bindings
            .iter()
            .filter_map(|binding| match (&binding.source, &binding.target) {
                (InputSource::Keyboard(key), BindingTarget::DrumLane(found))
                    if *found == target_lane =>
                {
                    Some(*key)
                }
                _ => None,
            })
            .collect()
    }
}

pub(crate) fn binding_triggered(
    source: &InputSource,
    keyboard: &ButtonInput<KeyCode>,
    midi_events: &[MidiNoteEvent],
) -> bool {
    match source {
        InputSource::Keyboard(key) => keyboard.just_pressed(*key),
        InputSource::MidiNote {
            device,
            note,
            channel,
        } => midi_events.iter().any(|event| {
            device.matches(&event.device_name)
                && event.note == *note
                && channel.is_none_or(|expected| expected == event.channel)
        }),
    }
}
