use std::time::Duration;

use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};
use bevy::winit::{UpdateMode, WinitSettings};

use dtxpt::input::{InputBindings, MidiInputState, SystemAction};

use crate::gameplay::run::RunState;

pub(crate) fn present_mode_has_vsync(mode: PresentMode) -> bool {
    !matches!(
        mode,
        PresentMode::AutoNoVsync | PresentMode::Immediate | PresentMode::Mailbox
    )
}

pub(crate) fn winit_settings_for_vsync(vsync: bool) -> WinitSettings {
    if vsync {
        let mode = UpdateMode::reactive(Duration::from_secs_f64(1.0 / 60.0));
        WinitSettings {
            focused_mode: mode,
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(1.0 / 60.0)),
        }
    } else {
        // Uncapped gameplay should not drop to reactive pacing when the terminal steals focus.
        WinitSettings::continuous()
    }
}

pub fn toggle_hotkeys(
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    bindings: Res<InputBindings>,
    mut run: ResMut<RunState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut winit: ResMut<WinitSettings>,
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
    *winit = winit_settings_for_vsync(present_mode_has_vsync(window.present_mode));
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
