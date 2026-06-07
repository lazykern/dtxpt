mod persist;
mod rebind;
mod rows;
mod ui;
mod values;

use bevy::prelude::*;

pub use persist::persist_runtime_config;
pub(crate) use ui::{
    refresh_settings_overlay, settings_row_interaction, setup_global, setup_settings_overlay,
    sync_settings_list_scroll,
};
pub use values::settings_overlay_toggle;

pub(crate) use rebind::settings_overlay_input;
pub(crate) use rows::{SettingCategory, SettingRow, filtered_settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RebindingTarget {
    Lane(usize),
    System(dtxpt::input::SystemAction),
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SettingsOverlay {
    pub(crate) search: String,
    pub(crate) selected: usize,
    pub(crate) category: SettingCategory,
    pub(crate) rebinding: Option<RebindingTarget>,
    pub(crate) lane_binding_cursor: usize,
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
    pub(crate) values: String,
}

#[derive(Resource, Default)]
pub(crate) struct SettingsScrollSync {
    pub(crate) selected: usize,
    pub(crate) category: SettingCategory,
    pub(crate) search: String,
}
