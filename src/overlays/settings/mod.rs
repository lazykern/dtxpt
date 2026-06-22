mod persist;
mod rebind;
mod rows;
mod ui;
mod values;

use bevy::prelude::*;

pub use persist::persist_runtime_config;
pub(crate) use ui::{
    refresh_settings_overlay, settings_row_interaction, setup_global, setup_settings_overlay,
    sync_settings_list_scroll, track_window_focus,
};
pub use values::settings_overlay_toggle;

pub(crate) use rebind::settings_overlay_input;
pub(crate) use rows::{SettingCategory, SettingRow, filtered_settings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RebindingTarget {
    Lane(dtxpt::input::BindingTarget),
    System(dtxpt::input::SystemAction),
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SettingsOverlay {
    pub(crate) search: String,
    pub(crate) selected: usize,
    pub(crate) category: SettingCategory,
    pub(crate) rebinding: Option<RebindingTarget>,
    pub(crate) lane_binding_cursor: usize,
    /// Set when the most recent rebind replaced a previously-bound
    /// target on the same instrument. UI surfaces it as a transient
    /// conflict message; cleared on any other rebind action.
    pub(crate) last_rebind_conflict: Option<String>,
}

#[derive(Component)]
pub(crate) struct SettingsList;

#[derive(Resource, Default)]
pub struct SettingsUiCache {
    pub(crate) search: String,
    pub(crate) selected: usize,
    pub(crate) category: SettingCategory,
    pub(crate) rebinding: Option<RebindingTarget>,
    pub(crate) lane_binding_cursor: usize,
    pub(crate) last_rebind_conflict: Option<String>,
    pub(crate) values: String,
}

#[derive(Resource, Default)]
pub(crate) struct SettingsScrollSync {
    pub(crate) selected: usize,
    pub(crate) category: SettingCategory,
    pub(crate) search: String,
}
