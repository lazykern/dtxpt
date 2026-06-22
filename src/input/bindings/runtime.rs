use bevy::prelude::*;

use super::keycodes::keycode_from_name;
use super::types::{
    BassLane, BindingInstrument, BindingTarget, DrumLane, GuitarLane, InputBinding,
    InputBindingConfig, InputSource, InputSourceConfig, SystemAction,
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

    /// Iterate bindings targeting the given instrument. Used by per-instrument
    /// gameplay plugins (drums / guitar / bass) to filter their view of the
    /// shared `InputBindings` resource.
    #[allow(private_interfaces)]
    pub fn for_instrument(
        &self,
        instrument: BindingInstrument,
    ) -> impl Iterator<Item = (&InputSource, &BindingTarget)> {
        self.bindings
            .iter()
            .filter(move |b| b.target.instrument() == instrument)
            .map(|b| (&b.source, &b.target))
    }

    pub fn lane_triggered(
        &self,
        lane: usize,
        keyboard: &ButtonInput<KeyCode>,
        midi_events: &[MidiNoteEvent],
    ) -> bool {
        self.lane_triggered_with_source(lane, keyboard, midi_events)
            .is_some()
    }

    /// Returns the first matching binding source for the lane, so callers can
    /// access the underlying MIDI event's `received_at` for input-timing
    /// compensation. Keyboard sources don't carry a per-event timestamp so
    /// the caller must capture one separately.
    pub fn lane_triggered_with_source<'a>(
        &'a self,
        lane: usize,
        keyboard: &ButtonInput<KeyCode>,
        midi_events: &'a [MidiNoteEvent],
    ) -> Option<LaneTriggerSource<'a>> {
        let target_lane = DrumLane::from_index(lane)?;
        for binding in &self.bindings {
            if binding.target != BindingTarget::DrumLane(target_lane) {
                continue;
            }
            match &binding.source {
                InputSource::Keyboard(key) => {
                    if keyboard.just_pressed(*key) {
                        return Some(LaneTriggerSource::Keyboard);
                    }
                }
                InputSource::MidiNote {
                    device,
                    note,
                    channel,
                } => {
                    for event in midi_events.iter() {
                        if device.matches(&event.device_name)
                            && event.note == *note
                            && channel.is_none_or(|expected| expected == event.channel)
                        {
                            return Some(LaneTriggerSource::Midi { event });
                        }
                    }
                }
            }
        }
        None
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

    /// Returns the first matching binding source for the guitar lane, or `None`.
    pub fn guitar_lane_triggered_with_source<'a>(
        &'a self,
        lane: GuitarLane,
        keyboard: &ButtonInput<KeyCode>,
        midi_events: &'a [MidiNoteEvent],
    ) -> Option<LaneTriggerSource<'a>> {
        for binding in &self.bindings {
            if binding.target != BindingTarget::GuitarLane(lane) {
                continue;
            }
            if let Some(source) = binding_source_triggered(&binding.source, keyboard, midi_events) {
                return Some(source);
            }
        }
        None
    }

    /// Returns the first matching binding source for the bass lane, or `None`.
    pub fn bass_lane_triggered_with_source<'a>(
        &'a self,
        lane: BassLane,
        keyboard: &ButtonInput<KeyCode>,
        midi_events: &'a [MidiNoteEvent],
    ) -> Option<LaneTriggerSource<'a>> {
        for binding in &self.bindings {
            if binding.target != BindingTarget::BassLane(lane) {
                continue;
            }
            if let Some(source) = binding_source_triggered(&binding.source, keyboard, midi_events) {
                return Some(source);
            }
        }
        None
    }

    pub fn guitar_lane_triggered(
        &self,
        lane: GuitarLane,
        keyboard: &ButtonInput<KeyCode>,
        midi_events: &[MidiNoteEvent],
    ) -> bool {
        self.guitar_lane_triggered_with_source(lane, keyboard, midi_events)
            .is_some()
    }

    pub fn bass_lane_triggered(
        &self,
        lane: BassLane,
        keyboard: &ButtonInput<KeyCode>,
        midi_events: &[MidiNoteEvent],
    ) -> bool {
        self.bass_lane_triggered_with_source(lane, keyboard, midi_events)
            .is_some()
    }
}

fn binding_source_triggered<'a>(
    source: &InputSource,
    keyboard: &ButtonInput<KeyCode>,
    midi_events: &'a [MidiNoteEvent],
) -> Option<LaneTriggerSource<'a>> {
    match source {
        InputSource::Keyboard(key) => keyboard
            .just_pressed(*key)
            .then_some(LaneTriggerSource::Keyboard),
        InputSource::MidiNote {
            device,
            note,
            channel,
        } => midi_events.iter().find_map(|event| {
            (device.matches(&event.device_name)
                && event.note == *note
                && channel.is_none_or(|expected| expected == event.channel))
            .then_some(LaneTriggerSource::Midi { event })
        }),
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

#[derive(Debug, Clone, Copy)]
pub enum LaneTriggerSource<'a> {
    Keyboard,
    Midi { event: &'a MidiNoteEvent },
}
