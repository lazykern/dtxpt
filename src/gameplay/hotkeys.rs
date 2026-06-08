use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};
use bevy::winit::WinitSettings;

use dtxpt::input::{InputBindings, MidiInputState, SystemAction};

use crate::gameplay::run::RunState;

pub(crate) fn present_mode_has_vsync(mode: PresentMode) -> bool {
    !matches!(
        mode,
        PresentMode::AutoNoVsync | PresentMode::Immediate | PresentMode::Mailbox
    )
}

/// Always run continuously. The `present_mode` on the Window (AutoVsync,
/// AutoNoVsync, Immediate, Mailbox) governs the actual render rate; decoupling
/// the update rate lets the game logic run as fast as the monitor on 120/144Hz
/// displays, smoothing note motion and shrinking input latency. The previous
/// `UpdateMode::reactive(1/60)` cap locked updates to 60Hz even on 144Hz
/// monitors, producing visible 16.67ms stutter steps in note motion.
pub(crate) fn winit_settings_for_vsync(_vsync: bool) -> WinitSettings {
    WinitSettings::continuous()
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
        PresentMode::Immediate
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
