use bevy::prelude::*;

use crate::gameplay::constants::{
    LANE_SPEED_STEP, MAX_LANE_SPEED, MIN_LANE_SPEED, TIMING_OFFSET_STEP,
};
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;
use dtxpt::input::bindings::SystemAction;
use dtxpt::input::{InputBindings, MidiInputState};

pub fn action_allowed_during_play(action: SystemAction, practice: bool) -> bool {
    match action {
        SystemAction::IncreaseSongRate
        | SystemAction::DecreaseSongRate
        | SystemAction::ResetSongRate
        | SystemAction::SeekForward
        | SystemAction::SeekBackward
        | SystemAction::SeekToPreviousMeasure
        | SystemAction::SeekToNextMeasure => practice,
        _ => true,
    }
}

pub fn play_mode_change_allowed_during_play(_active_practice: Option<bool>) -> bool {
    // Allowed any time. Changing practice / per-lane / auto_mode mid-run
    // resets the run state (counters, gauge) so the new mode takes effect
    // cleanly from the current chart position.
    true
}

pub fn song_rate_change_allowed_during_play(active_practice: Option<bool>) -> bool {
    match active_practice {
        None => true,
        Some(practice) => practice,
    }
}

pub fn adjust_timing_offset(
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    bindings: Res<InputBindings>,
    mut run: ResMut<RunState>,
) {
    let mut changed = false;
    if bindings.action_just_pressed(
        SystemAction::DecreaseTimingOffset,
        &keyboard,
        &midi.note_on_events,
    ) {
        run.timing_offset -= TIMING_OFFSET_STEP;
        changed = true;
    }
    if bindings.action_just_pressed(
        SystemAction::IncreaseTimingOffset,
        &keyboard,
        &midi.note_on_events,
    ) {
        run.timing_offset += TIMING_OFFSET_STEP;
        changed = true;
    }
    if bindings.action_just_pressed(
        SystemAction::ResetTimingOffset,
        &keyboard,
        &midi.note_on_events,
    ) {
        run.timing_offset = 0.0;
        changed = true;
    }
    if changed {
        info!("timing offset set to {:+.0} ms", run.timing_offset * 1000.0);
    }
}

pub fn adjust_lane_speed(
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    bindings: Res<InputBindings>,
    layout: Res<PlayfieldLayout>,
    mut run: ResMut<RunState>,
) {
    let mut changed = false;
    if bindings.action_just_pressed(
        SystemAction::DecreaseLaneSpeed,
        &keyboard,
        &midi.note_on_events,
    ) {
        run.lane_speed = (run.lane_speed - LANE_SPEED_STEP).clamp(MIN_LANE_SPEED, MAX_LANE_SPEED);
        changed = true;
    }
    if bindings.action_just_pressed(
        SystemAction::IncreaseLaneSpeed,
        &keyboard,
        &midi.note_on_events,
    ) {
        run.lane_speed = (run.lane_speed + LANE_SPEED_STEP).clamp(MIN_LANE_SPEED, MAX_LANE_SPEED);
        changed = true;
    }
    if bindings.action_just_pressed(
        SystemAction::ResetLaneSpeed,
        &keyboard,
        &midi.note_on_events,
    ) {
        run.lane_speed = 1.0;
        changed = true;
    }
    if changed {
        info!(
            "lane speed set to {:.2}x ({:.1} px/s)",
            run.lane_speed,
            layout.scroll_px_per_sec(run.lane_speed)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_mode_blocks_practice_tools() {
        assert!(!action_allowed_during_play(SystemAction::SeekForward, false));
        assert!(!song_rate_change_allowed_during_play(Some(false)));
    }

    #[test]
    fn play_mode_change_allowed_in_any_state() {
        assert!(play_mode_change_allowed_during_play(None));
        assert!(play_mode_change_allowed_during_play(Some(false)));
        assert!(play_mode_change_allowed_during_play(Some(true)));
    }

    #[test]
    fn practice_mode_allows_practice_tools() {
        assert!(action_allowed_during_play(
            SystemAction::IncreaseSongRate,
            true
        ));
        assert!(song_rate_change_allowed_during_play(Some(true)));
    }
}
