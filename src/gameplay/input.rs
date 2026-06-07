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

pub(crate) fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    pause_state: Res<State<PauseState>>,
    mut chart: ResMut<Chart>,
    mut run: ResMut<RunState>,
    mut commands: Commands,
    layout: Res<PlayfieldLayout>,
    clock: Res<ChartClock>,
    mut hit_audio: LaneHitAudio,
    bindings: Res<InputBindings>,
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
        return;
    }

    let elapsed = run.elapsed;
    for lane in 0..LANES.len() {
        let triggered = bindings.lane_triggered(lane, &keyboard, &midi.note_on_events);
        if !triggered {
            continue;
        }

        flash_lane_receptor(lane, &mut flashes.p0());
        keyboard_viz::flash_key_cap(lane, &mut flashes.p1());

        process_lane_hit(
            lane,
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
