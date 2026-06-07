use bevy::prelude::*;

use crate::app::markers::SettingsOverlayScreen;
use crate::app::state::OverlayState;
use crate::overlays::settings::{
    SettingsScrollSync, SettingsUiCache, refresh_settings_overlay, settings_overlay_input,
    settings_overlay_toggle, settings_row_interaction, setup_settings_overlay,
    sync_settings_list_scroll,
};
use crate::screens::common::cleanup_screen;

pub struct OverlaysPlugin;

impl Plugin for OverlaysPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsUiCache>()
            .init_resource::<SettingsScrollSync>()
            .add_systems(OnEnter(OverlayState::Settings), setup_settings_overlay)
            .add_systems(
                OnExit(OverlayState::Settings),
                cleanup_screen::<SettingsOverlayScreen>,
            )
            .add_systems(Update, settings_overlay_toggle)
            .add_systems(
                Update,
                (
                    settings_overlay_input,
                    settings_row_interaction.after(settings_overlay_input),
                    refresh_settings_overlay.after(settings_row_interaction),
                )
                    .run_if(in_state(OverlayState::Settings)),
            )
            .add_systems(
                PostUpdate,
                sync_settings_list_scroll
                    .after(refresh_settings_overlay)
                    .after(bevy::ui::UiSystems::PostLayout)
                    .run_if(in_state(OverlayState::Settings)),
            );
    }
}
