use bevy::{asset::Handle, prelude::*};
use bevy_kira_audio::prelude::*;

use dtxpt::chart::dtx::channels::is_dtx_stick_se_channel;
use dtxpt::input::lanes::{
    DTX_CH_HH_CLOSE, DTX_CH_HH_OPEN, DTX_CH_LP, DTX_CH_SE_HH, HH_TRACKED_WAV_CAP, LANES,
    POLYPHONIC_VOICES,
};

use super::super::mix::*;
use super::super::sound_bank::*;
use super::state::{ActiveSounds, AudioFrame, GameRng, MetronomeActive, TrackedAudioHandle};
use super::transport::combined_playback_rate;

pub(crate) fn collect_active_drum_handles(active: &ActiveSounds) -> Vec<Handle<AudioInstance>> {
    let mut handles = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for lane in &active.per_lane {
        for tracked in lane {
            if seen.insert(tracked.handle.clone()) {
                handles.push(tracked.handle.clone());
            }
        }
    }
    for voices in active.voice_pools.values() {
        for slot in &voices.slots {
            if let Some(tracked) = slot
                && seen.insert(tracked.handle.clone())
            {
                handles.push(tracked.handle.clone());
            }
        }
    }
    handles
}

pub(crate) fn stop_active_drums(
    audio_instances: &mut Assets<AudioInstance>,
    active: &mut ActiveSounds,
) {
    for handle in collect_active_drum_handles(active) {
        if let Some(instance) = audio_instances.get_mut(&handle) {
            instance.stop(instant_audio_tween());
        }
    }
    for lane in &mut active.per_lane {
        lane.clear();
    }
    for voices in active.voice_pools.values_mut() {
        for slot in &mut voices.slots {
            *slot = None;
        }
    }
    active.hh_tracked_wavs.clear();
    active.last_hh_channel = None;
}

pub(crate) fn pause_active_drums(
    audio_instances: &mut Assets<AudioInstance>,
    active: &ActiveSounds,
) {
    for handle in collect_active_drum_handles(active) {
        if let Some(instance) = audio_instances.get_mut(&handle) {
            instance.pause(instant_audio_tween());
        }
    }
}

pub(crate) fn resume_active_drums(
    audio_instances: &mut Assets<AudioInstance>,
    active: &ActiveSounds,
) {
    for handle in collect_active_drum_handles(active) {
        if let Some(instance) = audio_instances.get_mut(&handle) {
            instance.resume(instant_audio_tween());
        }
    }
}

pub(crate) fn next_voice_pool_slot(voices: &mut super::state::WavVoices, max_voices: usize) -> usize {
    let max_voices = max_voices.clamp(1, POLYPHONIC_VOICES);
    let slot = voices.next % max_voices;
    voices.next = (slot + 1) % max_voices;
    slot
}

pub(crate) fn assign_voice_pool_slot(
    voices: &mut super::state::WavVoices,
    slot: usize,
    handle: Handle<AudioInstance>,
    frame: u64,
) -> Option<TrackedAudioHandle> {
    let previous = voices.slots[slot].take();
    voices.slots[slot] = Some(TrackedAudioHandle {
        handle,
        born_frame: frame,
    });
    previous
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn active_voice_pool_slots(
    voices: &super::state::WavVoices,
    max_voices: usize,
) -> usize {
    let max_voices = max_voices.clamp(1, POLYPHONIC_VOICES);
    voices.slots[..max_voices]
        .iter()
        .filter(|slot| slot.is_some())
        .count()
}

fn normalize_hh_channel(channel: u32) -> u32 {
    if channel == DTX_CH_SE_HH {
        DTX_CH_HH_CLOSE
    } else {
        channel
    }
}

fn should_choke_hh(channel: u32, last_hh_channel: Option<u32>, lp_muting: bool) -> bool {
    let channel = normalize_hh_channel(channel);
    if last_hh_channel == Some(DTX_CH_HH_OPEN) {
        return false;
    }
    channel == DTX_CH_HH_CLOSE || (lp_muting && channel == DTX_CH_LP)
}

fn is_hh_tracked_channel(channel: u32) -> bool {
    matches!(
        normalize_hh_channel(channel),
        DTX_CH_HH_CLOSE | DTX_CH_HH_OPEN
    )
}

fn track_hh_wav(active: &mut ActiveSounds, wav_id: u32) {
    if active.hh_tracked_wavs.len() >= HH_TRACKED_WAV_CAP {
        active.hh_tracked_wavs.remove(0);
    }
    if !active.hh_tracked_wavs.contains(&wav_id) {
        active.hh_tracked_wavs.push(wav_id);
    }
}

fn choke_hh_wavs(active: &mut ActiveSounds, audio_instances: &mut Assets<AudioInstance>) {
    let wav_ids: Vec<u32> = active.hh_tracked_wavs.drain(..).collect();
    for wav_id in wav_ids {
        stop_wav_instant(wav_id, audio_instances, active);
    }
}

/// Play drum sound: per-WAV polyphony + DTXMania-style HH close/LP choke.
pub(crate) fn play_drum_sound(
    wav_id: Option<u32>,
    channel: u32,
    lane: usize,
    playback_rate: Option<f64>,
    song_rate: f32,
    frame: u64,
    lp_muting: bool,
    sound_bank: &SoundBank,
    mix: &AudioMix,
    audio: &Audio,
    audio_instances: &mut Assets<AudioInstance>,
    active: &mut ActiveSounds,
) {
    let Some(id) = wav_id else { return };

    if should_choke_hh(channel, active.last_hh_channel, lp_muting) {
        choke_hh_wavs(active, audio_instances);
    }

    if let Some(instance_handle) = play_wav(
        id,
        combined_playback_rate(song_rate, playback_rate),
        None,
        frame,
        sound_bank,
        mix,
        MixKind::Drums,
        audio,
        audio_instances,
        Some(active),
    ) {
        active.per_lane[lane].push(TrackedAudioHandle {
            handle: instance_handle,
            born_frame: frame,
        });
    }

    if is_hh_tracked_channel(channel) {
        active.last_hh_channel = Some(normalize_hh_channel(channel));
        track_hh_wav(active, id);
    }
}

pub(crate) fn play_auto_se_sound(
    wav_id: u32,
    channel: u32,
    song_rate: f32,
    frame: u64,
    sound_bank: &SoundBank,
    mix: &AudioMix,
    audio: &Audio,
    audio_instances: &mut Assets<AudioInstance>,
    active: &mut ActiveSounds,
) {
    if is_dtx_stick_se_channel(channel) {
        let index = (channel - 0x61) as usize;
        if let Some(previous_wav) = active.last_muting_se_wav[index] {
            stop_wav(previous_wav, audio_instances, active);
        }
        active.last_muting_se_wav[index] = Some(wav_id);
    }

    let _ = play_wav(
        wav_id,
        combined_playback_rate(song_rate, None),
        None,
        frame,
        sound_bank,
        mix,
        MixKind::Drums,
        audio,
        audio_instances,
        Some(active),
    );
}

pub(crate) fn play_wav(
    wav_id: u32,
    playback_rate: Option<f64>,
    start_from: Option<f64>,
    frame: u64,
    sound_bank: &SoundBank,
    mix: &AudioMix,
    kind: MixKind,
    audio: &Audio,
    audio_instances: &mut Assets<AudioInstance>,
    active: Option<&mut ActiveSounds>,
) -> Option<Handle<AudioInstance>> {
    let wav = sound_bank.wavs.get(&wav_id)?;
    let mut command = audio.play(wav.handle.clone());
    command.with_volume(mix.volume_db(wav.volume, kind));
    command.with_panning(dtx_pan_to_kira(wav.pan));
    if let Some(playback_rate) = playback_rate {
        command.with_playback_rate(playback_rate);
    }
    if let Some(start_from) = start_from {
        command.start_from(start_from);
    }
    let instance_handle = command.handle();
    if let Some(active) = active {
        let voices = active.voice_pools.entry(wav_id).or_default();
        let max_voices = wav.role.max_voices();
        let slot = next_voice_pool_slot(voices, max_voices);
        if let Some(previous) = assign_voice_pool_slot(voices, slot, instance_handle.clone(), frame)
            && let Some(inst) = audio_instances.get_mut(&previous.handle)
        {
            inst.stop(instant_audio_tween());
        }
    }
    Some(instance_handle)
}

fn rng_next_u32(rng: &mut GameRng) -> u32 {
    let x = &mut rng.0;
    *x ^= *x << 13;
    *x ^= *x >> 17;
    *x ^= *x << 5;
    *x
}

pub fn rng_next_usize(rng: &mut GameRng) -> usize {
    rng_next_u32(rng) as usize
}

pub(crate) fn dtx_bad_playback_rate(rng: &mut GameRng) -> f64 {
    let x = rng_next_u32(rng);
    let magnitude = ((x % 3) + 1) as f64 * 0.07;
    let sign = if (x & 0x08) == 0 { -1.0 } else { 1.0 };
    1.0 + magnitude * sign
}

pub(crate) fn log_active_audio_snapshot(
    tag: &str,
    active: &ActiveSounds,
    audio_instances: &Assets<AudioInstance>,
) {
    let lane_counts = active
        .per_lane
        .iter()
        .enumerate()
        .filter(|&(_lane, handles)| !handles.is_empty())
        .map(|(lane, handles)| format!("{}:{}", LANES[lane].label, handles.len()))
        .collect::<Vec<_>>();
    let wav_counts = active
        .voice_pools
        .iter()
        .map(|(wav_id, voices)| {
            let live = voices.slots.iter().filter(|slot| slot.is_some()).count();
            format!("{:02X}:{}", wav_id, live)
        })
        .collect::<Vec<_>>();
    let live_instances = active
        .voice_pools
        .values()
        .flat_map(|voices| voices.slots.iter())
        .flatten()
        .filter(|tracked| is_audio_instance_active(&tracked.handle, audio_instances))
        .count();
    info!(
        "audio snapshot {} live={} lanes=[{}] wavs=[{}] muting_se={:?}",
        tag,
        live_instances,
        lane_counts.join(", "),
        wav_counts.join(", "),
        active.last_muting_se_wav
    );
}

fn stop_wav_instant(
    wav_id: u32,
    audio_instances: &mut Assets<AudioInstance>,
    active: &mut ActiveSounds,
) {
    let Some(voices) = active.voice_pools.get_mut(&wav_id) else {
        return;
    };
    for slot in voices.slots.iter_mut() {
        if let Some(tracked) = slot.take()
            && let Some(inst) = audio_instances.get_mut(&tracked.handle)
        {
            inst.stop(instant_audio_tween());
        }
    }
}

fn stop_wav(wav_id: u32, audio_instances: &mut Assets<AudioInstance>, active: &mut ActiveSounds) {
    stop_wav_instant(wav_id, audio_instances, active);
}

/// Periodically prune finished audio instances so tracking doesn't leak.
pub fn cleanup_active_sounds(
    mut active: ResMut<ActiveSounds>,
    mut metronome_active: ResMut<MetronomeActive>,
    frame: Res<AudioFrame>,
    audio_instances: Res<Assets<AudioInstance>>,
) {
    let current_frame = frame.0;
    for lane in 0..10 {
        active.per_lane[lane].retain(|tracked| {
            tracked.born_frame + 2 >= current_frame
                || is_audio_instance_active(&tracked.handle, &audio_instances)
        });
    }
    active.voice_pools.retain(|_, voices| {
        for slot in voices.slots.iter_mut() {
            *slot = slot.take().filter(|tracked| {
                tracked.born_frame + 2 >= current_frame
                    || is_audio_instance_active(&tracked.handle, &audio_instances)
            });
        }
        voices.slots.iter().any(|slot| slot.is_some())
    });
    metronome_active
        .instances
        .retain(|handle| is_audio_instance_active(handle, &audio_instances));
}

fn is_audio_instance_active(
    handle: &Handle<AudioInstance>,
    audio_instances: &Assets<AudioInstance>,
) -> bool {
    audio_instances.get(handle).is_some_and(|inst| {
        !matches!(
            inst.state(),
            bevy_kira_audio::PlaybackState::Stopped
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lp_chokes_hh_only_when_muting_enabled() {
        assert!(should_choke_hh(DTX_CH_LP, Some(DTX_CH_HH_CLOSE), true));
        assert!(!should_choke_hh(DTX_CH_LP, Some(DTX_CH_HH_CLOSE), false));
        assert!(should_choke_hh(DTX_CH_HH_CLOSE, Some(DTX_CH_HH_CLOSE), false));
    }

    #[test]
    fn collect_active_drum_handles_deduplicates() {
        let handle = Handle::<AudioInstance>::default();
        let tracked = TrackedAudioHandle {
            handle: handle.clone(),
            born_frame: 0,
        };
        let mut active = ActiveSounds::default();
        active.per_lane[0].push(tracked.clone());
        active.voice_pools.insert(
            1,
            super::super::state::WavVoices {
                next: 0,
                slots: [Some(tracked), None, None, None],
            },
        );
        assert_eq!(collect_active_drum_handles(&active).len(), 1);
    }

    #[test]
    fn monophonic_roles_cap_at_one_active_slot() {
        use dtxpt::chart::WavRole;

        let handle = Handle::<AudioInstance>::default();
        for role in [WavRole::Bgm, WavRole::Se] {
            let max_voices = role.max_voices();
            assert_eq!(max_voices, 1);
            let mut voices = super::super::state::WavVoices::default();
            for frame in 0..3 {
                let slot = next_voice_pool_slot(&mut voices, max_voices);
                assign_voice_pool_slot(&mut voices, slot, handle.clone(), frame);
            }
            assert_eq!(active_voice_pool_slots(&voices, max_voices), 1);
        }
    }

    #[test]
    fn drum_role_keeps_four_active_slots() {
        use dtxpt::chart::WavRole;

        let max_voices = WavRole::Drum.max_voices();
        assert_eq!(max_voices, POLYPHONIC_VOICES);
        let handle = Handle::<AudioInstance>::default();
        let mut voices = super::super::state::WavVoices::default();
        for frame in 0..4 {
            let slot = next_voice_pool_slot(&mut voices, max_voices);
            assign_voice_pool_slot(&mut voices, slot, handle.clone(), frame);
        }
        assert_eq!(active_voice_pool_slots(&voices, max_voices), 4);
        let slot = next_voice_pool_slot(&mut voices, max_voices);
        assign_voice_pool_slot(&mut voices, slot, handle.clone(), 4);
        assert_eq!(active_voice_pool_slots(&voices, max_voices), 4);
    }

    #[test]
    fn role_caps_match_wav_role_max_voices() {
        use dtxpt::chart::WavRole;

        assert_eq!(WavRole::Drum.max_voices(), POLYPHONIC_VOICES);
        assert_eq!(WavRole::Bgm.max_voices(), 1);
        assert_eq!(WavRole::Se.max_voices(), 1);
        assert_eq!(WavRole::Guitar.max_voices(), 2);
        assert_eq!(WavRole::Bass.max_voices(), 2);
    }
}
