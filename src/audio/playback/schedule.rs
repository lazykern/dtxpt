use bevy::{asset::Handle, prelude::*};
use bevy_kira_audio::prelude::*;

use dtxpt::chart::dtx::channels::is_dtx_stick_se_channel;
use dtxpt::chart::{
    Chart, ChartNote, ScheduledAudioKind, chart_bgm_start_time, chart_notes_complete,
    should_suppress_metronome_beat,
};
use dtxpt::input::lanes::dtx_override_se_to_lane;

use crate::app::state::{PauseState, is_paused};
use crate::gameplay::clock::ChartClock;
use crate::gameplay::run::RunState;

use super::super::mix::*;
use super::super::sound_bank::*;
use super::state::{
    ActiveSounds, AudioFrame, BgmInstance, MetronomeActive, MetronomeSounds,
};
use super::transport::combined_playback_rate;
use super::voices::{play_auto_se_sound, play_drum_sound, play_wav};

pub(crate) fn stop_metronome_instances(
    metronome_active: &mut MetronomeActive,
    audio_instances: &mut Assets<AudioInstance>,
) {
    for handle in metronome_active.instances.drain(..) {
        if let Some(instance) = audio_instances.get_mut(&handle) {
            instance.stop(instant_audio_tween());
        }
    }
}

pub(crate) fn schedule_metronome(
    mut chart: ResMut<Chart>,
    pause_state: Res<State<PauseState>>,
    run: Res<RunState>,
    clock: Res<ChartClock>,
    metronome: Res<MetronomeSounds>,
    mix: Res<AudioMix>,
    audio: Res<Audio>,
    mut metronome_active: ResMut<MetronomeActive>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if is_paused(pause_state.get()) || run.finished || !run.metronome_sound {
        return;
    }

    // Warmup is chart time [-WARMUP_SECS, 0). No clicks during "starts in…" / READY.
    if clock.audio_elapsed < 0.0 || clock.visual_elapsed < 0.0 {
        stop_metronome_instances(&mut metronome_active, &mut audio_instances);
        return;
    }

    // Fire when the beat line crosses the judge line (same clock as note rendering).
    let elapsed = clock.visual_elapsed;
    let bgm_time = chart_bgm_start_time(&chart);
    let stick_se_times: Vec<f32> = chart
        .scheduled_audio
        .iter()
        .filter_map(|event| match event.kind {
            ScheduledAudioKind::AutoSe { channel } if is_dtx_stick_se_channel(channel) => {
                Some(event.time)
            }
            _ => None,
        })
        .collect();
    for beat in chart.metronome_beats.iter_mut() {
        if beat.fired || elapsed < beat.time {
            continue;
        }
        if should_suppress_metronome_beat(beat.time, bgm_time, &stick_se_times) {
            beat.fired = true;
            continue;
        }
        let handle = if beat.downbeat {
            metronome.downbeat.clone()
        } else {
            metronome.beat.clone()
        };
        let instance = audio.play(handle).with_volume(mix.master_db()).handle();
        metronome_active.instances.push(instance);
        beat.fired = true;
    }
}

pub(crate) fn schedule_auto_se(
    mut commands: Commands,
    mut chart: ResMut<Chart>,
    pause_state: Res<State<PauseState>>,
    run: Res<RunState>,
    clock: Res<ChartClock>,
    frame: Res<AudioFrame>,
    sound_bank: Res<SoundBank>,
    mix: Res<AudioMix>,
    audio: Res<Audio>,
    bgm_instance: Option<Res<BgmInstance>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut active: ResMut<ActiveSounds>,
) {
    if is_paused(pause_state.get()) || run.finished {
        return;
    }

    let elapsed = clock.audio_elapsed;
    for event in chart.scheduled_audio.iter_mut() {
        if event.fired || elapsed < event.time {
            continue;
        }
        match event.kind {
            ScheduledAudioKind::Bgm => {
                if bgm_instance.is_some() {
                    event.fired = true;
                } else if let Some(handle) = play_wav(
                    event.wav_id,
                    combined_playback_rate(run.song_playback_rate, None),
                    None,
                    frame.0,
                    &sound_bank,
                    &mix,
                    MixKind::Bgm,
                    &audio,
                    &mut audio_instances,
                    None,
                ) {
                    let dtx_volume = sound_bank
                        .wavs
                        .get(&event.wav_id)
                        .map(|wav| wav.volume)
                        .unwrap_or(100);
                    commands.insert_resource(BgmInstance {
                        handle,
                        start_time: event.time,
                        dtx_volume,
                    });
                    info!("BGM started at chart time {:.3}s", event.time);
                    event.fired = true;
                }
                // else: wav not loaded yet, retry next frame
            }
            ScheduledAudioKind::AutoSe { channel } => {
                if sound_bank.wavs.contains_key(&event.wav_id) {
                    if let Some(lane) = dtx_override_se_to_lane(channel) {
                        play_drum_sound(
                            Some(event.wav_id),
                            channel,
                            lane,
                            None,
                            run.song_playback_rate,
                            frame.0,
                            run.lp_muting,
                            &sound_bank,
                            &mix,
                            &audio,
                            &mut audio_instances,
                            &mut active,
                        );
                    } else {
                        play_auto_se_sound(
                            event.wav_id,
                            channel,
                            run.song_playback_rate,
                            frame.0,
                            &sound_bank,
                            &mix,
                            &audio,
                            &mut audio_instances,
                            &mut active,
                        );
                    }
                    event.fired = true;
                }
                // else: wav not loaded yet, retry next frame
            }
        }
    }
}

fn is_bgm_active(bgm: &BgmInstance, audio_instances: &Assets<AudioInstance>) -> bool {
    audio_instances
        .get(&bgm.handle)
        .is_some_and(|inst| !matches!(inst.state(), PlaybackState::Stopped))
}

pub(crate) fn should_finish_run(started: bool, notes_complete: bool, bgm_active: bool) -> bool {
    started && notes_complete && !bgm_active
}

pub(crate) fn update_finished_state(
    run: &mut RunState,
    notes: &[ChartNote],
    bgm_instance: Option<&BgmInstance>,
    audio_instances: &Assets<AudioInstance>,
) {
    if run.failed {
        run.finished = true;
        return;
    }
    let bgm_active = bgm_instance.is_some_and(|bgm| is_bgm_active(bgm, audio_instances));
    run.finished = should_finish_run(run.started, chart_notes_complete(notes), bgm_active);
}

pub(crate) fn check_song_finished(
    chart: Res<Chart>,
    bgm_instance: Option<Res<BgmInstance>>,
    audio_instances: Res<Assets<AudioInstance>>,
    mut run: ResMut<RunState>,
) {
    update_finished_state(
        &mut run,
        &chart.notes,
        bgm_instance.as_deref(),
        &audio_instances,
    );
}

pub(crate) fn start_bgm_at_chart_time(
    commands: &mut Commands,
    chart: &mut Chart,
    target: f32,
    song_rate: f32,
    frame: u64,
    sound_bank: &SoundBank,
    mix: &AudioMix,
    audio: &Audio,
    audio_instances: &mut Assets<AudioInstance>,
) -> Option<Handle<AudioInstance>> {
    let bgm_index = chart
        .scheduled_audio
        .iter()
        .position(|event| matches!(event.kind, ScheduledAudioKind::Bgm))?;
    let bgm_time = chart.scheduled_audio[bgm_index].time;
    if target < bgm_time {
        return None;
    }
    let wav_id = chart.scheduled_audio[bgm_index].wav_id;
    if !sound_bank.wavs.contains_key(&wav_id) {
        return None;
    }

    let file_pos = (target - bgm_time).max(0.0) as f64;
    let dtx_volume = sound_bank
        .wavs
        .get(&wav_id)
        .map(|wav| wav.volume)
        .unwrap_or(100);
    let handle = play_wav(
        wav_id,
        combined_playback_rate(song_rate, None),
        Some(file_pos),
        frame,
        sound_bank,
        mix,
        MixKind::Bgm,
        audio,
        audio_instances,
        None,
    )?;
    commands.insert_resource(BgmInstance {
        handle: handle.clone(),
        start_time: bgm_time,
        dtx_volume,
    });
    chart.scheduled_audio[bgm_index].fired = true;
    info!(
        "BGM seek-started at chart {:.3}s (file {:.3}s)",
        target, file_pos
    );
    Some(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_finish_run_waits_for_bgm_after_notes_complete() {
        assert!(!should_finish_run(true, true, true));
        assert!(should_finish_run(true, true, false));
        assert!(!should_finish_run(true, false, false));
        assert!(!should_finish_run(false, true, false));
    }
}
