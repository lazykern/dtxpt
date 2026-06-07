use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};

use dtxpt::input::{InputBindings, MidiInputState, SystemAction};

use crate::gameplay::run::RunState;

pub(crate) fn present_mode_has_vsync(mode: PresentMode) -> bool {
    !matches!(
        mode,
        PresentMode::AutoNoVsync | PresentMode::Immediate | PresentMode::Mailbox
    )
}

pub fn toggle_hotkeys(
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    bindings: Res<InputBindings>,
    mut run: ResMut<RunState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if bindings.action_just_pressed(
        SystemAction::ToggleMetronomeSound,
        &keyboard,
        &midi.note_on_events,
    ) {
        run.metronome_sound = !run.metronome_sound;
        info!(
            "metronome sound {}",
            if run.metronome_sound {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    if bindings.action_just_pressed(
        SystemAction::ToggleDebugHud,
        &keyboard,
        &midi.note_on_events,
    ) {
        run.show_debug_hud = !run.show_debug_hud;
        info!(
            "debug hud {}",
            if run.show_debug_hud {
                "shown"
            } else {
                "hidden"
            }
        );
    }

    if !bindings.action_just_pressed(SystemAction::ToggleVsync, &keyboard, &midi.note_on_events) {
        return;
    }

    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    window.present_mode = if present_mode_has_vsync(window.present_mode) {
        PresentMode::AutoNoVsync
    } else {
        PresentMode::AutoVsync
    };
    info!(
        "present mode set to {:?} ({})",
        window.present_mode,
        if present_mode_has_vsync(window.present_mode) {
            "vsync on"
        } else {
            "vsync off"
        }
    );
}
