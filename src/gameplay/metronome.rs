use bevy::prelude::*;
use dtxpt::chart::MetronomeBeat;
use kira::Frame;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

use crate::app::markers::{GameplayEntity, MetronomeLineVisual};
use crate::gameplay::clock::ChartClock;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;

#[derive(Resource, Default)]
pub struct MetronomeLineStream {
    next_spawn_index: usize,
}

impl MetronomeLineStream {
    pub fn reset(&mut self) {
        self.next_spawn_index = 0;
    }

    pub fn align_to_time(&mut self, beats: &[MetronomeBeat], min_time: f32) {
        self.next_spawn_index = beats
            .iter()
            .position(|beat| beat.time >= min_time)
            .unwrap_or(beats.len());
    }

    pub fn spawn_visible_through(
        &mut self,
        commands: &mut Commands,
        beats: &[MetronomeBeat],
        layout: &PlayfieldLayout,
        clock: &ChartClock,
        run: &RunState,
        max_time: f32,
    ) {
        while self.next_spawn_index < beats.len() {
            let beat = &beats[self.next_spawn_index];
            if beat.time > max_time {
                break;
            }
            spawn_metronome_line(commands, layout, clock, run, self.next_spawn_index, beat);
            self.next_spawn_index += 1;
        }
    }
}

pub fn make_metronome_click(freq_hz: f32, duration_ms: f32, gain: f32) -> StaticSoundData {
    let sample_rate = 44_100;
    let num_frames = (sample_rate as f32 * duration_ms / 1000.0).round() as usize;
    let duration_secs = duration_ms / 1000.0;
    let frames: Vec<Frame> = (0..num_frames)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let env = (1.0 - t / duration_secs).max(0.0);
            let sample = (std::f32::consts::TAU * freq_hz * t).sin() * env * gain;
            Frame::from_mono(sample)
        })
        .collect();
    StaticSoundData {
        sample_rate,
        frames: frames.into(),
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}

pub fn spawn_metronome_line(
    commands: &mut Commands,
    layout: &PlayfieldLayout,
    clock: &ChartClock,
    run: &RunState,
    beat_index: usize,
    beat: &MetronomeBeat,
) {
    let (height, color) = if beat.downbeat {
        (
            layout.metronome_line_height * 1.4,
            Color::srgba(0.95, 0.95, 1.0, 0.55),
        )
    } else {
        (
            layout.metronome_line_height,
            Color::srgba(0.65, 0.72, 0.85, 0.28),
        )
    };
    commands.spawn((
        Sprite::from_color(color, Vec2::new(layout.judge_line_width, height)),
        Transform::from_xyz(
            0.0,
            layout.note_y(beat.time, clock.visual_smoothed, run.lane_speed),
            0.0,
        ),
        MetronomeLineVisual { beat_index },
        GameplayEntity,
    ));
}
