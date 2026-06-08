use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use dtxpt::input::{InputBindings, MidiInputState, SystemAction};

use crate::config::GameConfig;
use crate::gameplay::run::RunState;

pub fn toggle_hotkeys(
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    bindings: Res<InputBindings>,
    mut run: ResMut<RunState>,
    mut config: ResMut<GameConfig>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut winit: ResMut<bevy::winit::WinitSettings>,
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

    if !bindings.action_just_pressed(SystemAction::CycleFpsCap, &keyboard, &midi.note_on_events) {
        return;
    }

    config.fps_cap = config.fps_cap.next();
    let cap = config.fps_cap;
    if let Ok(mut window) = windows.single_mut() {
        window.present_mode = cap.present_mode();
    }
    *winit = cap.winit_settings();
    info!("fps cap set to {} ({:?})", cap.label(), cap.present_mode());
}
