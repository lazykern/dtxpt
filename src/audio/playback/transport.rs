use bevy::{asset::Handle, prelude::*};
use bevy_kira_audio::prelude::*;

use dtxpt::chart::{
    Chart, ChartTiming, clamp_chart_time, reconcile_notes_for_restart, reconcile_notes_for_seek,
    reconcile_scheduled_for_time, reconcile_metronome_for_time,
};
use dtxpt::input::SystemAction;

use crate::app::markers::{MetronomeLineVisual, NoteVisual};
use crate::app::state::{PauseState, is_paused};
use crate::gameplay::clock::ChartClock;
use crate::gameplay::constants::*;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::live_tuning::action_allowed_during_play;
use crate::gameplay::metronome::spawn_metronome_lines;
use crate::gameplay::rendering::notes::{despawn_note_visuals, spawn_note_visuals};
use crate::gameplay::run::RunState;

use super::super::mix::*;
use super::super::sound_bank::*;
use super::schedule::{start_bgm_at_chart_time, stop_metronome_instances};
use super::state::{
    ActiveSounds, AudioFrame, BgmInstance, BoundInput, MetronomeActive, PlaybackAudio,
    RestartResume,
};
use super::voices::{log_active_audio_snapshot, pause_active_drums, resume_active_drums, stop_active_drums};

pub(crate) fn adjust_audio_mix(
    input: BoundInput,
    mut mix: ResMut<AudioMix>,
    bgm: Option<Res<BgmInstance>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let mut changed = false;
    if input.action_just_pressed(SystemAction::DecreaseMasterVolume) {
        mix.master = (mix.master - VOLUME_STEP).clamp(0.0, 1.0);
        changed = true;
    }
    if input.action_just_pressed(SystemAction::IncreaseMasterVolume) {
        mix.master = (mix.master + VOLUME_STEP).clamp(0.0, 1.0);
        changed = true;
    }
    if input.action_just_pressed(SystemAction::DecreaseBgmVolume) {
        mix.bgm = (mix.bgm - VOLUME_STEP).clamp(0.0, 1.0);
        changed = true;
    }
    if input.action_just_pressed(SystemAction::IncreaseBgmVolume) {
        mix.bgm = (mix.bgm + VOLUME_STEP).clamp(0.0, 1.0);
        changed = true;
    }
    if input.action_just_pressed(SystemAction::DecreaseDrumVolume) {
        mix.drums = (mix.drums - VOLUME_STEP).clamp(0.0, 1.0);
        changed = true;
    }
    if input.action_just_pressed(SystemAction::IncreaseDrumVolume) {
        mix.drums = (mix.drums + VOLUME_STEP).clamp(0.0, 1.0);
        changed = true;
    }
    if !changed {
        return;
    }

    info!(
        "volumes: master {:.0}%  BGM {:.0}%  drums {:.0}%",
        mix.master * 100.0,
        mix.bgm * 100.0,
        mix.drums * 100.0,
    );

    if let Some(bgm) = bgm
        && let Some(inst) = audio_instances.get_mut(&bgm.handle) {
            inst.set_decibels(
                mix.volume_db(bgm.dtx_volume, MixKind::Bgm),
                instant_audio_tween(),
            );
        }
}

pub(crate) fn sync_elapsed_from_audio(
    audio_instances: Res<Assets<AudioInstance>>,
    bgm_instance: Option<Res<BgmInstance>>,
    pause_state: Res<State<PauseState>>,
    mut run: ResMut<RunState>,
    mut clock: ResMut<ChartClock>,
    time: Res<Time>,
) {
    if is_paused(pause_state.get()) || run.finished {
        return;
    }

    let frame_dt = time.delta_secs();
    let previous_audio = clock.audio_elapsed;
    let next_audio = if let Some(ref bgm) = bgm_instance {
        if let Some(pos) = audio_instances
            .get(&bgm.handle)
            .and_then(|inst| inst.state().position())
        {
            // BGM audio position + scheduled BGM chip time = raw chart clock.
            bgm.start_time + pos as f32
        } else {
            // Fallback: still queued, advance smoothly.
            previous_audio + frame_dt * run.song_playback_rate
        }
    } else {
        previous_audio + frame_dt * run.song_playback_rate
    };

    clock.audio_elapsed = next_audio;
    clock.audio_step_ms = (clock.audio_elapsed - previous_audio) * 1000.0;
    clock.judgement_elapsed = clock.audio_elapsed + run.timing_offset;

    let target_visual = clock.judgement_elapsed;
    let visual_correction =
        if (target_visual - clock.visual_elapsed).abs() > VISUAL_SNAP_THRESHOLD_SECS {
            let correction = target_visual - clock.visual_elapsed;
            clock.visual_elapsed = target_visual;
            correction
        } else {
            clock.visual_elapsed += frame_dt;
            let drift = target_visual - clock.visual_elapsed;
            let catchup = (VISUAL_CORRECTION_GAIN * frame_dt).clamp(0.0, 1.0);
            let correction = drift * catchup;
            clock.visual_elapsed += correction;
            correction
        };

    clock.visual_drift_ms = (target_visual - clock.visual_elapsed) * 1000.0;
    clock.visual_correction_ms = visual_correction * 1000.0;

    run.raw_elapsed = clock.audio_elapsed;
    run.elapsed = clock.judgement_elapsed;
    if run.elapsed >= 0.0 {
        run.started = true;
    }
    run.judgement_timer.tick(time.delta());
}

pub(crate) fn stop_all_playback(
    commands: &mut Commands,
    audio_instances: &mut Assets<AudioInstance>,
    bgm_instance: Option<Res<BgmInstance>>,
    active: &mut ActiveSounds,
    metronome_active: &mut MetronomeActive,
) {
    if let Some(bgm) = bgm_instance {
        if let Some(instance) = audio_instances.get_mut(&bgm.handle) {
            instance.stop(AudioTween::default());
        }
        commands.remove_resource::<BgmInstance>();
    }

    stop_active_drums(audio_instances, active);
    stop_metronome_instances(metronome_active, audio_instances);
    active.voice_pools.clear();
    active.last_muting_se_wav = [None; 5];
}

pub(crate) fn set_playback_paused(
    paused: bool,
    bgm: Option<&Handle<AudioInstance>>,
    active: &mut ActiveSounds,
    metronome_active: &mut MetronomeActive,
    audio_instances: &mut Assets<AudioInstance>,
) {
    if paused {
        if let Some(handle) = bgm
            && let Some(instance) = audio_instances.get_mut(handle) {
                instance.pause(instant_audio_tween());
            }
        pause_active_drums(audio_instances, active);
        stop_metronome_instances(metronome_active, audio_instances);
    } else {
        if let Some(handle) = bgm
            && let Some(instance) = audio_instances.get_mut(handle) {
                instance.resume(instant_audio_tween());
            }
        resume_active_drums(audio_instances, active);
    }
}

pub(crate) fn set_clock_to_time(clock: &mut ChartClock, run: &mut RunState, target: f32) {
    clock.audio_elapsed = target;
    clock.judgement_elapsed = target + run.timing_offset;
    clock.visual_elapsed = clock.judgement_elapsed;
    clock.audio_step_ms = 0.0;
    clock.visual_drift_ms = 0.0;
    clock.visual_correction_ms = 0.0;
    run.raw_elapsed = target;
    run.elapsed = clock.judgement_elapsed;
    run.started = run.elapsed >= 0.0;
}

pub(crate) fn respawn_playfield_visuals(
    commands: &mut Commands,
    chart: &Chart,
    layout: &PlayfieldLayout,
    clock: &ChartClock,
    run: &RunState,
    visuals: &mut ParamSet<(
        Query<Entity, With<NoteVisual>>,
        Query<Entity, With<MetronomeLineVisual>>,
    )>,
) {
    despawn_note_visuals(commands, visuals.p0().iter());
    for entity in visuals.p1().iter().collect::<Vec<_>>() {
        commands.entity(entity).despawn();
    }
    spawn_metronome_lines(commands, chart, layout, clock, run);
    spawn_note_visuals(commands, chart, layout, clock, run);
}

pub(crate) fn seek_playback_to_time(
    target: f32,
    was_paused: bool,
    chart: &mut Chart,
    run: &mut RunState,
    clock: &mut ChartClock,
    layout: &PlayfieldLayout,
    commands: &mut Commands,
    audio_instances: &mut Assets<AudioInstance>,
    bgm_instance: Option<Res<BgmInstance>>,
    active: &mut ActiveSounds,
    metronome_active: &mut MetronomeActive,
    sound_bank: &SoundBank,
    mix: &AudioMix,
    audio: &Audio,
    frame: u64,
    visuals: &mut ParamSet<(
        Query<Entity, With<NoteVisual>>,
        Query<Entity, With<MetronomeLineVisual>>,
    )>,
) {
    stop_all_playback(
        commands,
        audio_instances,
        bgm_instance,
        active,
        metronome_active,
    );
    reconcile_notes_for_seek(&mut chart.notes, target);
    reconcile_scheduled_for_time(&mut chart.scheduled_audio, target);
    reconcile_metronome_for_time(&mut chart.metronome_beats, target);
    set_clock_to_time(clock, run, target);
    if !was_paused {
        start_bgm_at_chart_time(
            commands,
            chart,
            target,
            run.song_playback_rate,
            frame,
            sound_bank,
            mix,
            audio,
            audio_instances,
        );
    }
    run.finished = false;
    respawn_playfield_visuals(commands, chart, layout, clock, run, visuals);
    info!(
        "seek complete target={:.3}s elapsed={:.3}s",
        target, run.elapsed
    );
}

pub(crate) fn restart_playback(
    chart: &mut Chart,
    run: &mut RunState,
    clock: &mut ChartClock,
    layout: &PlayfieldLayout,
    commands: &mut Commands,
    audio_instances: &mut Assets<AudioInstance>,
    bgm_instance: Option<Res<BgmInstance>>,
    active: &mut ActiveSounds,
    metronome_active: &mut MetronomeActive,
    visuals: &mut ParamSet<(
        Query<Entity, With<NoteVisual>>,
        Query<Entity, With<MetronomeLineVisual>>,
    )>,
) {
    log_active_audio_snapshot("before-restart", active, audio_instances);
    stop_all_playback(
        commands,
        audio_instances,
        bgm_instance,
        active,
        metronome_active,
    );
    log_active_audio_snapshot("after-stop-before-reset", active, audio_instances);

    let timing_offset = run.timing_offset;
    let lane_speed = run.lane_speed;
    let song_playback_rate = run.song_playback_rate;
    let metronome_sound = run.metronome_sound;
    let show_debug_hud = run.show_debug_hud;
    let play_mode = run.play_mode;

    reconcile_notes_for_restart(&mut chart.notes);
    for event in chart.scheduled_audio.iter_mut() {
        event.fired = false;
    }
    for beat in chart.metronome_beats.iter_mut() {
        beat.fired = false;
    }

    *run = RunState::default();
    run.timing_offset = timing_offset;
    run.lane_speed = lane_speed;
    run.song_playback_rate = song_playback_rate;
    run.metronome_sound = metronome_sound;
    run.show_debug_hud = show_debug_hud;
    run.play_mode = play_mode;
    clock.reset(run.timing_offset);
    run.raw_elapsed = clock.audio_elapsed;
    run.elapsed = clock.judgement_elapsed;

    respawn_playfield_visuals(commands, chart, layout, clock, run, visuals);
    info!(
        "restart complete; notes reset={} scheduled_audio reset={}",
        chart.notes.len(),
        chart.scheduled_audio.len()
    );
}

pub(crate) fn combined_playback_rate(song_rate: f32, modifier: Option<f64>) -> Option<f64> {
    let rate = song_rate as f64 * modifier.unwrap_or(1.0);
    if (rate - 1.0).abs() < f64::EPSILON {
        None
    } else {
        Some(rate)
    }
}

#[derive(Resource, Default)]
pub struct RestartGestureState {
    last_tap_at: Option<f32>,
    hold_started_at: Option<f32>,
    hold_fired: bool,
}

fn restart_gesture_triggered(
    now: f32,
    state: &mut RestartGestureState,
    keys: &[KeyCode],
    input: &BoundInput,
) -> bool {
    if keys.is_empty() {
        state.last_tap_at = None;
        state.hold_started_at = None;
        state.hold_fired = false;
        return false;
    }

    let any_just_pressed = keys.iter().any(|key| input.key_just_pressed(*key));
    let any_pressed = keys.iter().any(|key| input.key_pressed(*key));

    if any_just_pressed {
        if let Some(last) = state.last_tap_at
            && now - last <= RESTART_DOUBLE_TAP_SECS {
                state.last_tap_at = None;
                state.hold_started_at = None;
                state.hold_fired = false;
                return true;
            }
        state.last_tap_at = Some(now);
        state.hold_started_at = Some(now);
        state.hold_fired = false;
    }

    if any_pressed {
        if let Some(started) = state.hold_started_at
            && !state.hold_fired && now - started >= RESTART_HOLD_SECS {
                state.hold_fired = true;
                state.last_tap_at = None;
                return true;
            }
    } else {
        state.hold_started_at = None;
        state.hold_fired = false;
    }

    false
}

pub(crate) fn restart_on_gesture(
    input: BoundInput,
    time: Res<Time>,
    mut gesture: ResMut<RestartGestureState>,
    mut resume: RestartResume,
    mut chart: ResMut<Chart>,
    mut run: ResMut<RunState>,
    mut clock: ResMut<ChartClock>,
    layout: Res<PlayfieldLayout>,
    mut commands: Commands,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    bgm_instance: Option<Res<BgmInstance>>,
    mut active: ResMut<ActiveSounds>,
    mut metronome_active: ResMut<MetronomeActive>,
    mut visuals: ParamSet<(
        Query<Entity, With<NoteVisual>>,
        Query<Entity, With<MetronomeLineVisual>>,
    )>,
) {
    let keys = input.keyboard_keys_for_action(SystemAction::RestartChart);
    if !restart_gesture_triggered(time.elapsed_secs(), &mut gesture, &keys, &input) {
        return;
    }

    let was_paused = is_paused(resume.pause_state.get());

    info!(
        "restart requested raw_elapsed={:.3}s elapsed={:.3}s offset_ms={:+.0}",
        run.raw_elapsed,
        run.elapsed,
        run.timing_offset * 1000.0
    );
    restart_playback(
        &mut chart,
        &mut run,
        &mut clock,
        &layout,
        &mut commands,
        &mut audio_instances,
        bgm_instance,
        &mut active,
        &mut metronome_active,
        &mut visuals,
    );

    if !was_paused {
        return;
    }

    resume.next_pause.set(PauseState::Running);
    let bgm_handle = start_bgm_at_chart_time(
        &mut commands,
        &mut chart,
        clock.audio_elapsed,
        run.song_playback_rate,
        resume.frame.0,
        &resume.playback.sound_bank,
        &resume.playback.mix,
        &resume.playback.audio,
        &mut audio_instances,
    );
    set_playback_paused(
        false,
        bgm_handle.as_ref(),
        &mut active,
        &mut metronome_active,
        &mut audio_instances,
    );
    info!("restart resumed playback from pause");
}

pub(crate) fn adjust_song_playback_rate(
    input: BoundInput,
    mut run: ResMut<RunState>,
    bgm_instance: Option<Res<BgmInstance>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let play_mode = run.play_mode;
    let mut changed = false;
    if input.action_just_pressed(SystemAction::DecreaseSongRate)
        && action_allowed_during_play(SystemAction::DecreaseSongRate, play_mode)
    {
        run.song_playback_rate = (run.song_playback_rate - SONG_RATE_STEP)
            .clamp(MIN_SONG_PLAYBACK_RATE, MAX_SONG_PLAYBACK_RATE);
        changed = true;
    }
    if input.action_just_pressed(SystemAction::ResetSongRate)
        && action_allowed_during_play(SystemAction::ResetSongRate, play_mode)
    {
        run.song_playback_rate = 1.0;
        changed = true;
    }
    if input.action_just_pressed(SystemAction::IncreaseSongRate)
        && action_allowed_during_play(SystemAction::IncreaseSongRate, play_mode)
    {
        run.song_playback_rate = (run.song_playback_rate + SONG_RATE_STEP)
            .clamp(MIN_SONG_PLAYBACK_RATE, MAX_SONG_PLAYBACK_RATE);
        changed = true;
    }

    if !changed {
        return;
    }

    if let Some(bgm) = bgm_instance
        && let Some(instance) = audio_instances.get_mut(&bgm.handle) {
            if let Some(rate) = combined_playback_rate(run.song_playback_rate, None) {
                instance.set_playback_rate(rate, instant_audio_tween());
            } else {
                instance.set_playback_rate(1.0, instant_audio_tween());
            }
        }

    info!("song playback rate set to {:.2}x", run.song_playback_rate);
}

pub(crate) fn playback_transport(
    input: BoundInput,
    timing: Res<ChartTiming>,
    pause_state: Res<State<PauseState>>,
    mut chart: ResMut<Chart>,
    mut run: ResMut<RunState>,
    mut clock: ResMut<ChartClock>,
    layout: Res<PlayfieldLayout>,
    mut commands: Commands,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    bgm_instance: Option<Res<BgmInstance>>,
    mut active: ResMut<ActiveSounds>,
    mut metronome_active: ResMut<MetronomeActive>,
    playback_audio: PlaybackAudio,
    frame: Res<AudioFrame>,
    mut visuals: ParamSet<(
        Query<Entity, With<NoteVisual>>,
        Query<Entity, With<MetronomeLineVisual>>,
    )>,
) {
    let play_mode = run.play_mode;
    let mut seek_target = None;
    if input.action_just_pressed(SystemAction::SeekForward)
        && action_allowed_during_play(SystemAction::SeekForward, play_mode)
    {
        seek_target = Some(clamp_chart_time(
            &timing,
            clock.audio_elapsed + SEEK_STEP_SECS,
            WARMUP_SECS,
        ));
    } else if input.action_just_pressed(SystemAction::SeekBackward)
        && action_allowed_during_play(SystemAction::SeekBackward, play_mode)
    {
        seek_target = Some(clamp_chart_time(
            &timing,
            clock.audio_elapsed - SEEK_STEP_SECS,
            WARMUP_SECS,
        ));
    } else if input.action_just_pressed(SystemAction::SeekToPreviousMeasure)
        && action_allowed_during_play(SystemAction::SeekToPreviousMeasure, play_mode)
    {
        let measure = timing
            .measure_at_time(clock.audio_elapsed)
            .saturating_sub(1);
        seek_target = Some(clamp_chart_time(
            &timing,
            timing.time_at_measure(measure),
            WARMUP_SECS,
        ));
    } else if input.action_just_pressed(SystemAction::SeekToNextMeasure)
        && action_allowed_during_play(SystemAction::SeekToNextMeasure, play_mode)
    {
        let measure = timing.measure_at_time(clock.audio_elapsed) + 1;
        seek_target = Some(clamp_chart_time(
            &timing,
            timing.time_at_measure(measure),
            WARMUP_SECS,
        ));
    }

    if let Some(target) = seek_target {
        info!(
            "seek requested from {:.3}s to {:.3}s",
            clock.audio_elapsed, target
        );
        seek_playback_to_time(
            target,
            is_paused(pause_state.get()),
            &mut chart,
            &mut run,
            &mut clock,
            &layout,
            &mut commands,
            &mut audio_instances,
            bgm_instance,
            &mut active,
            &mut metronome_active,
            &playback_audio.sound_bank,
            &playback_audio.mix,
            &playback_audio.audio,
            frame.0,
            &mut visuals,
        );
    }
}
