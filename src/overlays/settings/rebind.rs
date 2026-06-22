#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy::winit::WinitSettings;

use dtxpt::input::{
    BassLane, BindingTarget, GuitarLane, InputBindings, MidiInputState, add_keyboard_binding,
    add_midi_binding, remove_target_binding_at, reset_system_keyboard_binding,
    set_system_keyboard_binding, target_binding_indices,
};

use crate::app::state::OverlayState;
use crate::audio::AudioMix;
use crate::config::{GameConfig, save_game_config};
use crate::gameplay::run::RunState;
use crate::ui::input::UiKeyRepeat;
use crate::ui::search_char;

use super::values::apply_setting_delta;
use super::{RebindingTarget, SettingRow, SettingsOverlay, filtered_settings};

pub(crate) fn settings_overlay_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    midi: Res<MidiInputState>,
    time: Res<Time>,
    mut repeat: Local<UiKeyRepeat>,
    mut overlay: ResMut<SettingsOverlay>,
    mut config: ResMut<GameConfig>,
    mut mix: ResMut<AudioMix>,
    mut bindings: ResMut<InputBindings>,
    run: Option<ResMut<RunState>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut winit: ResMut<WinitSettings>,
    mut next_overlay: ResMut<NextState<OverlayState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        overlay.rebinding = None;
        next_overlay.set(OverlayState::None);
        return;
    }

    let mut needs_redraw = false;

    if let Some(target) = overlay.rebinding.clone() {
        let mut saved = false;
        match target {
            RebindingTarget::Lane(lane) => {
                for key in keyboard.get_just_pressed() {
                    if matches!(
                        *key,
                        KeyCode::Escape | KeyCode::F1 | KeyCode::Enter | KeyCode::Tab
                    ) {
                        continue;
                    }
                    if let Ok(result) =
                        add_keyboard_binding(&mut config.bindings, lane.clone(), *key)
                    {
                        saved = true;
                        overlay.last_rebind_conflict = result.replaced.map(|prev| {
                            format!("Replaced previous binding to {}", target_label(&prev))
                        });
                    }
                    overlay.rebinding = None;
                    break;
                }
                if overlay.rebinding.is_some()
                    && let Some(event) = midi.note_on_events.first()
                {
                    if let Ok(result) = add_midi_binding(
                        &mut config.bindings,
                        lane.clone(),
                        &event.device_name,
                        event.channel,
                        event.note,
                    ) {
                        saved = true;
                        overlay.last_rebind_conflict = result.replaced.map(|prev| {
                            format!("Replaced previous binding to {}", target_label(&prev))
                        });
                    }
                    overlay.rebinding = None;
                }
            }
            RebindingTarget::System(action) => {
                for key in keyboard.get_just_pressed() {
                    if matches!(
                        *key,
                        KeyCode::Escape | KeyCode::F1 | KeyCode::Enter | KeyCode::Tab
                    ) {
                        continue;
                    }
                    if set_system_keyboard_binding(&mut config.bindings, action, *key).is_ok() {
                        saved = true;
                    }
                    overlay.rebinding = None;
                    break;
                }
            }
        }
        if saved {
            *bindings = InputBindings::from_config(&config.bindings);
            if let Err(err) = save_game_config(&config) {
                warn!("failed to save config: {err}");
            }
        }
        return;
    }

    if overlay.search.trim().is_empty() && keyboard.just_pressed(KeyCode::Tab) {
        overlay.category = overlay.category.next();
        overlay.selected = 0;
        needs_redraw = true;
    }
    let rows = filtered_settings(&overlay.search, overlay.category);
    let selected_lane = rows.get(overlay.selected).and_then(|row| match row {
        SettingRow::LaneKey(lane) => Some(lane.clone()),
        _ => None,
    });

    let selected_system_action = rows.get(overlay.selected).and_then(|row| match row {
        SettingRow::SystemAction(action) => Some(*action),
        _ => None,
    });

    if overlay.search.trim().is_empty()
        && (keyboard.just_pressed(KeyCode::Backspace) || keyboard.just_pressed(KeyCode::Delete))
    {
        let mut changed = false;
        if let Some(lane) = selected_lane.clone() {
            let entry_count = target_binding_indices(&config.bindings, lane.clone()).len();
            if entry_count > 0 {
                let cursor = overlay
                    .lane_binding_cursor
                    .min(entry_count.saturating_sub(1));
                changed = remove_target_binding_at(&mut config.bindings, lane.clone(), cursor);
                if changed {
                    let remaining = target_binding_indices(&config.bindings, lane).len();
                    overlay.lane_binding_cursor =
                        overlay.lane_binding_cursor.min(remaining.saturating_sub(1));
                }
            }
        } else if let Some(action) = selected_system_action {
            reset_system_keyboard_binding(&mut config.bindings, action);
            changed = true;
        }
        if changed {
            *bindings = InputBindings::from_config(&config.bindings);
            if let Err(err) = save_game_config(&config) {
                warn!("failed to save config: {err}");
            }
            needs_redraw = true;
        }
    } else if keyboard.just_pressed(KeyCode::Backspace) {
        overlay.search.pop();
        overlay.selected = overlay.selected.min(
            filtered_settings(&overlay.search, overlay.category)
                .len()
                .saturating_sub(1),
        );
        needs_redraw = true;
    } else if keyboard.just_pressed(KeyCode::Delete) {
        overlay.search.clear();
        overlay.selected = 0;
        needs_redraw = true;
    }
    for key in keyboard.get_just_pressed() {
        if let Some(ch) = search_char(*key) {
            overlay.search.push(ch);
            overlay.selected = 0;
            needs_redraw = true;
        }
    }

    let rows = filtered_settings(&overlay.search, overlay.category);
    if rows.is_empty() {
        overlay.selected = 0;
    } else {
        let repeat_key = repeat.update(
            &keyboard,
            &time,
            &[
                KeyCode::ArrowDown,
                KeyCode::ArrowUp,
                KeyCode::ArrowRight,
                KeyCode::ArrowLeft,
            ],
        );
        if repeat_key == Some(KeyCode::ArrowDown) {
            overlay.selected = (overlay.selected + 1).min(rows.len() - 1);
            overlay.lane_binding_cursor = 0;
            needs_redraw = true;
        }
        if repeat_key == Some(KeyCode::ArrowUp) {
            overlay.selected = overlay.selected.saturating_sub(1);
            overlay.lane_binding_cursor = 0;
            needs_redraw = true;
        }

        let lane_cursor_delta = if let Some(lane) = selected_lane.clone() {
            let entry_count = target_binding_indices(&config.bindings, lane).len();
            if entry_count > 0 {
                if repeat_key == Some(KeyCode::ArrowRight) {
                    Some(1)
                } else if repeat_key == Some(KeyCode::ArrowLeft) {
                    Some(-1)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(delta) = lane_cursor_delta {
            let entry_count = target_binding_indices(
                &config.bindings,
                selected_lane.clone().expect("lane row selected"),
            )
            .len();
            let next = if delta > 0 {
                (overlay.lane_binding_cursor + 1).min(entry_count.saturating_sub(1))
            } else {
                overlay.lane_binding_cursor.saturating_sub(1)
            };
            if next != overlay.lane_binding_cursor {
                overlay.lane_binding_cursor = next;
                needs_redraw = true;
            }
        }

        let delta = if lane_cursor_delta.is_some() {
            0.0
        } else if repeat_key == Some(KeyCode::ArrowRight) {
            1.0
        } else if repeat_key == Some(KeyCode::ArrowLeft) {
            -1.0
        } else {
            0.0
        };
        if keyboard.just_pressed(KeyCode::Enter) {
            match rows.get(overlay.selected) {
                Some(SettingRow::LaneKey(lane)) => {
                    overlay.rebinding = Some(RebindingTarget::Lane(lane.clone()));
                    needs_redraw = true;
                }
                Some(SettingRow::SystemAction(action)) => {
                    overlay.rebinding = Some(RebindingTarget::System(*action));
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        if delta != 0.0 {
            let mut run = run;
            let Some(selected) = rows.get(overlay.selected).cloned() else {
                return;
            };
            let changed = match windows.single_mut() {
                Ok(mut window) => apply_setting_delta(
                    &selected,
                    delta,
                    &mut config,
                    &mut mix,
                    run.as_deref_mut(),
                    Some(window.as_mut()),
                    Some(&mut winit),
                ),
                Err(_) => apply_setting_delta(
                    &selected,
                    delta,
                    &mut config,
                    &mut mix,
                    run.as_deref_mut(),
                    None,
                    Some(&mut winit),
                ),
            };
            if changed {
                if let Err(err) = save_game_config(&config) {
                    warn!("failed to save config: {err}");
                }
                needs_redraw = true;
            }
        }
    }

    let _ = needs_redraw;
}

/// Format a binding target for the rebind conflict message.
fn target_label(target: &BindingTarget) -> String {
    match target {
        BindingTarget::DrumLane(lane) => {
            format!("drum {}", dtxpt::input::lanes::LANES[lane.index()].label)
        }
        BindingTarget::GuitarLane(lane) => format!(
            "guitar {}",
            match lane {
                GuitarLane::R => "R",
                GuitarLane::G => "G",
                GuitarLane::B => "B",
                GuitarLane::Y => "Y",
                GuitarLane::P => "P",
                _ => "?",
            }
        ),
        BindingTarget::BassLane(lane) => format!(
            "bass {}",
            match lane {
                BassLane::R => "R",
                BassLane::G => "G",
                BassLane::B => "B",
                BassLane::P => "P",
                _ => "?",
            }
        ),
        BindingTarget::System(action) => format!("system {}", action.label()),
    }
}

#[cfg(test)]
mod tests {
    use dtxpt::input::{BassLane, BindingTarget, DrumLane, GuitarLane, SystemAction};

    use super::target_label;

    #[test]
    fn target_label_formats_drum_guitar_bass_and_system() {
        assert_eq!(
            target_label(&BindingTarget::DrumLane(DrumLane::Bd)),
            "drum BD"
        );
        assert_eq!(
            target_label(&BindingTarget::GuitarLane(GuitarLane::R)),
            "guitar R"
        );
        assert_eq!(
            target_label(&BindingTarget::BassLane(BassLane::P)),
            "bass P"
        );
        assert_eq!(
            target_label(&BindingTarget::System(SystemAction::PauseToggle)),
            "system Pause / resume"
        );
    }
}
