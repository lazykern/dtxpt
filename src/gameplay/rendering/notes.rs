use bevy::prelude::*;

use dtxpt::chart::{ChartNote, NoteState};
use dtxpt::input::lanes::LANES;

use crate::app::markers::{GameplayEntity, NoteVisual};
use crate::gameplay::clock::ChartClock;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;

#[derive(Resource, Default)]
pub struct NoteVisualStream {
    next_spawn_index: usize,
}

#[derive(Resource, Default)]
pub struct PlayfieldVisualStreams {
    pub metronome: crate::gameplay::metronome::MetronomeLineStream,
    pub notes: NoteVisualStream,
}

impl NoteVisualStream {
    pub fn reset(&mut self) {
        self.next_spawn_index = 0;
    }

    pub fn align_to_time(&mut self, notes: &[ChartNote], min_time: f32) {
        self.next_spawn_index = notes
            .iter()
            .position(|note| note.state == NoteState::Pending && note.time >= min_time)
            .unwrap_or(notes.len());
    }

    pub fn spawn_visible_through(
        &mut self,
        commands: &mut Commands,
        notes: &[ChartNote],
        layout: &PlayfieldLayout,
        clock: &ChartClock,
        run: &RunState,
        max_time: f32,
    ) {
        while self.next_spawn_index < notes.len() {
            let note = &notes[self.next_spawn_index];
            if note.state != NoteState::Pending {
                self.next_spawn_index += 1;
                continue;
            }
            if note.time > max_time {
                break;
            }
            spawn_note_visual(commands, layout, clock, run, self.next_spawn_index, note);
            self.next_spawn_index += 1;
        }
    }
}

pub fn spawn_note_visual(
    commands: &mut Commands,
    layout: &PlayfieldLayout,
    clock: &ChartClock,
    run: &RunState,
    note_index: usize,
    note: &ChartNote,
) {
    commands.spawn((
        Sprite::from_color(
            LANES[note.lane].color,
            Vec2::new(layout.note_bar_w(), layout.note_h),
        ),
        Transform::from_xyz(
            layout.lane_x(note.lane),
            layout.note_y(note.time, clock.predicted_visual, run.lane_speed),
            1.0,
        ),
        NoteVisual { note_index },
        GameplayEntity,
    ));
}

pub fn despawn_note_visuals(commands: &mut Commands, notes: impl Iterator<Item = Entity>) {
    for entity in notes {
        commands.entity(entity).despawn();
    }
}
