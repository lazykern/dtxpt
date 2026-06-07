use bevy::{asset::Handle, ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::{AudioSource, prelude::*};

use dtxpt::input::lanes::POLYPHONIC_VOICES;
use dtxpt::input::{InputBindings, MidiInputState, SystemAction};

use crate::app::state::PauseState;

use super::super::mix::*;
use super::super::sound_bank::*;

#[derive(Resource)]
pub(crate) struct BgmInstance {
    pub handle: Handle<AudioInstance>,
    pub start_time: f32,
    pub dtx_volume: i32,
}

#[derive(Resource)]
pub(crate) struct MetronomeSounds {
    pub(crate) downbeat: Handle<AudioSource>,
    pub(crate) beat: Handle<AudioSource>,
}

#[derive(SystemParam)]
pub(crate) struct PlaybackAudio<'w> {
    pub sound_bank: Res<'w, SoundBank>,
    pub mix: Res<'w, AudioMix>,
    pub audio: Res<'w, Audio>,
}

#[derive(SystemParam)]
pub(crate) struct RestartResume<'w> {
    pub(super) pause_state: Res<'w, State<PauseState>>,
    pub(super) next_pause: ResMut<'w, NextState<PauseState>>,
    pub(super) frame: Res<'w, AudioFrame>,
    pub(super) playback: PlaybackAudio<'w>,
}

#[derive(SystemParam)]
pub struct BoundInput<'w> {
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    midi: Res<'w, MidiInputState>,
    bindings: Res<'w, InputBindings>,
}

impl BoundInput<'_> {
    pub fn action_just_pressed(&self, action: SystemAction) -> bool {
        self.bindings
            .action_just_pressed(action, &self.keyboard, &self.midi.note_on_events)
    }

    pub fn keyboard_keys_for_action(&self, action: SystemAction) -> Vec<KeyCode> {
        self.bindings.keyboard_keys_for_action(action)
    }

    pub fn key_just_pressed(&self, key: KeyCode) -> bool {
        self.keyboard.just_pressed(key)
    }

    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keyboard.pressed(key)
    }
}

#[derive(Resource)]
pub struct GameRng(pub(super) u32);

impl Default for GameRng {
    fn default() -> Self {
        Self(0x1234ABCD)
    }
}

/// Track active AudioInstance handles per lane (debug/restart bookkeeping).
#[derive(Clone)]
pub(crate) struct TrackedAudioHandle {
    pub(super) handle: Handle<AudioInstance>,
    pub(super) born_frame: u64,
}

/// Per-WAV round-robin voice pool (DTXMania `nPolyphonicSounds` model).
#[derive(Default)]
pub(crate) struct WavVoices {
    pub(super) next: usize,
    pub(super) slots: [Option<TrackedAudioHandle>; POLYPHONIC_VOICES],
}

#[derive(Resource, Default)]
pub struct ActiveSounds {
    pub(super) voice_pools: std::collections::HashMap<u32, WavVoices>,
    pub(super) per_lane: [Vec<TrackedAudioHandle>; 10],
    pub(super) last_muting_se_wav: [Option<u32>; 5],
    pub(super) hh_tracked_wavs: Vec<u32>,
    pub(super) last_hh_channel: Option<u32>,
}

#[derive(Resource, Default)]
pub struct MetronomeActive {
    pub(crate) instances: Vec<Handle<AudioInstance>>,
}

#[derive(Resource, Default)]
pub struct AudioFrame(pub u64);

pub fn advance_audio_frame(mut frame: ResMut<AudioFrame>) {
    frame.0 = frame.0.wrapping_add(1);
}
