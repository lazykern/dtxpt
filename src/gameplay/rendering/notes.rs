use bevy::prelude::*;

use dtxpt::chart::Chart;
use dtxpt::input::lanes::LANES;

use crate::app::markers::{GameplayEntity, NoteVisual};
use crate::gameplay::clock::ChartClock;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;

pub fn spawn_note_visuals(
    commands: &mut Commands,
    chart: &Chart,
    layout: &PlayfieldLayout,
    clock: &ChartClock,
    run: &RunState,
) {
    for (note_index, note) in chart.notes.iter().enumerate() {
        commands.spawn((
            Sprite::from_color(
                LANES[note.lane].color,
                Vec2::new(layout.note_bar_w(), layout.note_h),
            ),
            Transform::from_xyz(
                layout.lane_x(note.lane),
                layout.note_y(note.time, clock.visual_elapsed, run.lane_speed),
                1.0,
            ),
            NoteVisual { note_index },
            GameplayEntity,
        ));
    }
}

pub fn despawn_note_visuals(commands: &mut Commands, notes: impl Iterator<Item = Entity>) {
    for entity in notes {
        commands.entity(entity).despawn();
    }
}
