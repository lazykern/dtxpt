#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::app::markers::{LaneReceptor, LaneReceptorFlash};
use crate::app::state::{PauseState, is_paused};
use crate::audio::*;
use crate::gameplay::clock::ChartClock;
use crate::gameplay::judgement::process_lane_hit;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::rendering::{keyboard_viz, playfield_viz::lane_receptor_color};
use crate::gameplay::run::RunState;
use dtxpt::chart::{Chart, chart_notes_complete};
use dtxpt::input::lanes::LANES;
use dtxpt::input::{InputBindings, MidiInputState};

#[derive(SystemParam)]
pub(crate) struct LaneHitAudio<'w> {
    frame: Res<'w, AudioFrame>,
    sound_bank: Res<'w, SoundBank>,
    mix: Res<'w, AudioMix>,
    audio: Res<'w, Audio>,
    audio_instances: ResMut<'w, Assets<AudioInstance>>,
    active: ResMut<'w, ActiveSounds>,
    rng: ResMut<'w, GameRng>,
}

#[derive(Debug, Clone, Copy)]
pub struct LaneHitEvent {
    pub lane: usize,
    /// Wall-clock seconds (Time::elapsed_secs) when the trigger was first observed.
    pub wall_time: f32,
}

#[derive(Resource, Default)]
pub struct PendingLaneInputs {
    pub events: Vec<LaneHitEvent>,
}

/// Capture lane triggers as early as possible in the frame, stamped with wall-clock time.
/// Runs after `sync_elapsed_from_audio` and `toggle_playback_pause` so it sees the
/// current pause state. Input state for lanes only — system actions (pause, restart, etc.)
/// still use their own detection paths in the original system order.
pub(crate) fn capture_lane_inputs(
    time: Res<Time>,
    pause_state: Res<State<PauseState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    bindings: Res<InputBindings>,
    mut pending: ResMut<PendingLaneInputs>,
) {
    if is_paused(pause_state.get()) {
        return;
    }
    let now = time.elapsed_secs();
    let triggered_midi = &midi.note_on_events;
    for lane in 0..LANES.len() {
        if bindings.lane_triggered(lane, &keyboard, triggered_midi) {
            pending.events.push(LaneHitEvent { lane, wall_time: now });
        }
    }
}

/// Drain captured lane hits. For each event, compute the audio clock at event time
/// by subtracting (now_wall - event_wall) * song_rate from the current audio clock.
/// This compensates for the per-frame processing delay between key observation
/// (early in the frame) and judgement processing (later in the same frame).
pub(crate) fn process_pending_lane_hits(
    time: Res<Time>,
    pause_state: Res<State<PauseState>>,
    mut chart: ResMut<Chart>,
    mut run: ResMut<RunState>,
    mut commands: Commands,
    layout: Res<PlayfieldLayout>,
    clock: Res<ChartClock>,
    mut hit_audio: LaneHitAudio,
    bindings: Res<InputBindings>,
    mut pending: ResMut<PendingLaneInputs>,
    mut flashes: ParamSet<(
        Query<(&LaneReceptor, &mut Sprite, &mut LaneReceptorFlash)>,
        Query<
            (
                &keyboard_viz::KeyCap,
                &mut Sprite,
                &mut keyboard_viz::KeyCapFlash,
            ),
            Without<keyboard_viz::KeyCapLabel>,
        >,
    )>,
) {
    if run.finished
        || run.failed
        || is_paused(pause_state.get())
        || chart_notes_complete(&chart.notes)
    {
        // discard any stale captures that snuck in across a state change
        pending.events.clear();
        return;
    }

    if pending.events.is_empty() {
        return;
    }

    let now_wall = time.elapsed_secs();
    let song_rate = run.song_playback_rate;
    let audio_now = clock.audio_elapsed;
    let timing_offset = run.timing_offset;

    let events = std::mem::take(&mut pending.events);
    for event in events {
        let wall_delta = (now_wall - event.wall_time).max(0.0);
        let audio_at_event = audio_now - wall_delta * song_rate;
        let elapsed = audio_at_event + timing_offset;

        flash_lane_receptor(event.lane, &mut flashes.p0());
        keyboard_viz::flash_key_cap(event.lane, &mut flashes.p1());

        process_lane_hit(
            event.lane,
            elapsed,
            &mut chart,
            &mut run,
            &mut commands,
            &layout,
            &clock,
            hit_audio.frame.0,
            &hit_audio.sound_bank,
            &hit_audio.mix,
            &hit_audio.audio,
            &mut hit_audio.audio_instances,
            &mut hit_audio.active,
            &mut hit_audio.rng,
            &mut flashes.p0(),
        );
    }
    // `bindings` is intentionally kept in the signature for future per-lane filtering;
    // the actual filtering is done at capture time so timing is preserved.
    let _ = bindings;
}

pub(crate) fn flash_lane_receptor(
    lane: usize,
    receptors: &mut Query<(&LaneReceptor, &mut Sprite, &mut LaneReceptorFlash)>,
) {
    use crate::gameplay::constants::LANE_RECEPTOR_FLASH_SECS;

    for (receptor, mut sprite, mut flash) in receptors.iter_mut() {
        if receptor.lane != lane {
            continue;
        }
        flash.timer = Timer::from_seconds(LANE_RECEPTOR_FLASH_SECS, TimerMode::Once);
        sprite.color = lane_receptor_color(lane, 1.0);
        break;
    }
}
