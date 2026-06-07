use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use bevy::winit::WinitSettings;

use dtxpt::input::lanes::LANES;

use crate::app::markers::{SettingsOverlayScreen, SettingsRowMarker};
use crate::audio::AudioMix;
use crate::config::{GameConfig, save_game_config};
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;
use crate::ui::fonts::{UiFonts, text_font};
use crate::ui::palette::*;
use crate::ui::scroll::{child_range_where, scroll_to_show_range_y};
use crate::ui::theme::SPACING_XS;
use crate::ui::theme::*;
use crate::ui::widgets::*;

use super::values::{apply_setting_delta, apply_vsync_setting};
use super::{
    RebindingTarget, SettingRow, SettingsList, SettingsOverlay, SettingsScrollSync,
    SettingsUiCache, filtered_settings,
};

pub fn setup_global(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut winit: ResMut<WinitSettings>,
    mut layout: ResMut<PlayfieldLayout>,
) {
    if let Ok(mut window) = windows.single_mut() {
        *layout = PlayfieldLayout::from_window(&window);
        apply_vsync_setting(&mut window, Some(&mut winit), config.vsync);
    }
    commands.spawn((Camera2d, IsDefaultUiCamera));

    match ron::ser::to_string(&*config) {
        Ok(config_preview) => info!("starter config: {config_preview}"),
        Err(err) => warn!("failed to serialize starter config: {err}"),
    }
}

pub(crate) fn setup_settings_overlay(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    mut cache: ResMut<SettingsUiCache>,
    mut scroll_sync: ResMut<SettingsScrollSync>,
    overlay: Res<SettingsOverlay>,
) {
    cache.selected = usize::MAX;
    cache.values.clear();
    scroll_sync.selected = overlay.selected.wrapping_add(1);
    scroll_sync.category = overlay.category;
    scroll_sync.search.clear();
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ZIndex(200),
        SettingsOverlayScreen,
        children![
            (
                Node {
                    width: percent(100),
                    height: percent(100),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(BG_OVERLAY),
                SettingsOverlayScreen,
            ),
            (
                Node {
                    width: px(920.0),
                    height: px(620.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(SPACING_LG)),
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::all(px(BORDER_RADIUS)),
                    row_gap: px(SPACING_MD),
                    ..default()
                },
                BackgroundColor(BG_SECONDARY),
                BorderColor::all(BORDER_SUBTLE),
                SettingsOverlayScreen,
                children![
                    (
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            (
                                Text::new("Settings"),
                                text_font(&fonts, FONT_HEADING),
                                TextColor(TEXT_ACCENT),
                            ),
                            (
                                Text::new("F1/Esc close  Tab category  ↑/↓ select  ←/→ adjust"),
                                text_font(&fonts, FONT_CAPTION),
                                TextColor(TEXT_MUTED),
                            ),
                        ],
                    ),
                    (scroll_list_node(), SettingsList, SettingsOverlayScreen),
                ],
            ),
        ],
    ));
}

pub(crate) fn refresh_settings_overlay(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    overlay: Res<SettingsOverlay>,
    config: Res<GameConfig>,
    mix: Res<AudioMix>,
    mut cache: ResMut<SettingsUiCache>,
    list_query: Query<Entity, With<SettingsList>>,
) {
    let values = settings_values_signature(&overlay, &config, &mix);
    if cache.search == overlay.search
        && cache.selected == overlay.selected
        && cache.category == overlay.category
        && cache.rebinding == overlay.rebinding
        && cache.lane_binding_cursor == overlay.lane_binding_cursor
        && cache.values == values
    {
        return;
    }
    cache.search = overlay.search.clone();
    cache.selected = overlay.selected;
    cache.category = overlay.category;
    cache.rebinding = overlay.rebinding;
    cache.lane_binding_cursor = overlay.lane_binding_cursor;
    cache.values = values;

    let Ok(list_entity) = list_query.single() else {
        return;
    };
    commands.entity(list_entity).despawn_children();

    let rows = filtered_settings(&overlay.search, overlay.category);
    let searching = !overlay.search.trim().is_empty();

    commands.entity(list_entity).with_children(|parent| {
        let header = if searching {
            format!("Search: {}", overlay.search)
        } else {
            format!("{} — Search: {}", overlay.category.label(), overlay.search)
        };
        parent.spawn((
            Text::new(header),
            text_font(&fonts, FONT_BODY),
            TextColor(TEXT_SECONDARY),
        ));

        if let Some(target) = overlay.rebinding {
            let prompt = match target {
                RebindingTarget::Lane(lane) => format!(
                    "Rebinding {} — press key or MIDI pad (Esc cancel)",
                    LANES[lane].label
                ),
                RebindingTarget::System(action) => {
                    format!("Rebinding {} — press key (Esc cancel)", action.label())
                }
            };
            parent.spawn((
                Text::new(prompt),
                text_font(&fonts, FONT_BODY),
                TextColor(WARNING),
            ));
        }

        if rows.is_empty() {
            parent.spawn((
                Text::new("No matching settings."),
                text_font(&fonts, FONT_BODY),
                TextColor(TEXT_MUTED),
            ));
            return;
        }

        let mut current_category = None;
        for (index, row) in rows.iter().enumerate() {
            if searching {
                let row_category = row.category();
                if current_category != Some(row_category) {
                    current_category = Some(row_category);
                    parent.spawn((
                        Text::new(format!("[{}]", row_category.label())),
                        text_font(&fonts, FONT_CAPTION),
                        TextColor(TEXT_ACCENT),
                    ));
                }
            }

            let selected = index == overlay.selected;
            let value = if selected {
                row.value_with_overlay(&config, &mix, &overlay)
            } else {
                row.value(&config, &mix)
            };
            let ratio = row.slider_ratio(&config, &mix);

            parent.spawn((
                Button,
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(SPACING_SM)),
                    margin: UiRect::bottom(px(SPACING_XS)),
                    border_radius: BorderRadius::all(px(6.0)),
                    border: UiRect::all(px(if selected { 2.0 } else { 1.0 })),
                    ..default()
                },
                BackgroundColor(if selected { CARD_SELECTED } else { CARD_NORMAL }),
                BorderColor::all(if selected {
                    BORDER_FOCUS
                } else {
                    BORDER_SUBTLE
                }),
                SettingsRowMarker { index },
                SettingsOverlayScreen,
                children![
                    (
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            (
                                Text::new(row.label()),
                                text_font(&fonts, FONT_BODY),
                                TextColor(TEXT_PRIMARY),
                            ),
                            (
                                Text::new(value.clone()),
                                text_font(&fonts, FONT_BODY),
                                TextColor(TEXT_ACCENT),
                            ),
                        ],
                    ),
                    setting_control_placeholder(
                        ratio,
                        row.is_toggle().then(|| row.toggle_value(&config)),
                    ),
                    (
                        Text::new(row.description()),
                        text_font(&fonts, FONT_CAPTION),
                        TextColor(TEXT_MUTED),
                    ),
                ],
            ));
        }
    });
}

fn settings_values_signature(
    overlay: &SettingsOverlay,
    config: &GameConfig,
    mix: &AudioMix,
) -> String {
    let rows = filtered_settings(&overlay.search, overlay.category);
    rows.into_iter()
        .enumerate()
        .map(|(index, row)| {
            if index == overlay.selected {
                row.value_with_overlay(config, mix, overlay)
            } else {
                row.value(config, mix)
            }
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn setting_control_placeholder(ratio: Option<f32>, toggle: Option<bool>) -> impl Bundle {
    let has_control = ratio.is_some() || toggle.is_some();
    let enabled = toggle.unwrap_or(false);
    let parent_width = if ratio.is_some() {
        240.0
    } else if toggle.is_some() {
        40.0
    } else {
        0.0
    };
    let parent_height = if ratio.is_some() {
        8.0
    } else if toggle.is_some() {
        22.0
    } else {
        0.0
    };
    let parent_bg = if toggle == Some(true) {
        ACCENT
    } else {
        BG_ELEVATED
    };
    let child_width = if let Some(ratio) = ratio {
        percent(ratio.clamp(0.0, 1.0) * 100.0)
    } else if toggle.is_some() {
        px(18.0)
    } else {
        px(0.0)
    };
    let child_height = if ratio.is_some() {
        percent(100)
    } else if toggle.is_some() {
        px(18.0)
    } else {
        px(0.0)
    };
    let child_margin = if toggle.is_some() {
        UiRect::left(px(if enabled { 18.0 } else { 2.0 }))
    } else {
        UiRect::ZERO
    };

    (
        Node {
            width: px(parent_width),
            height: px(parent_height),
            margin: UiRect::top(px(if has_control { SPACING_XS } else { 0.0 })),
            padding: UiRect::all(px(if toggle.is_some() { 2.0 } else { 0.0 })),
            border_radius: BorderRadius::all(px(if toggle.is_some() { 11.0 } else { 4.0 })),
            ..default()
        },
        BackgroundColor(parent_bg),
        children![(
            Node {
                width: child_width,
                height: child_height,
                margin: child_margin,
                border_radius: BorderRadius::all(px(if toggle.is_some() { 9.0 } else { 4.0 })),
                ..default()
            },
            BackgroundColor(if toggle.is_some() {
                TEXT_PRIMARY
            } else {
                ACCENT
            }),
        )],
    )
}

pub(crate) fn settings_row_interaction(
    mut overlay: ResMut<SettingsOverlay>,
    mut config: ResMut<GameConfig>,
    mut mix: ResMut<AudioMix>,
    run: Option<ResMut<RunState>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut winit: ResMut<WinitSettings>,
    rows_query: Query<(&Interaction, &SettingsRowMarker), Changed<Interaction>>,
) {
    let mut run = run;
    for (interaction, marker) in &rows_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let rows = filtered_settings(&overlay.search, overlay.category);
        if marker.index >= rows.len() {
            continue;
        }
        overlay.selected = marker.index;
        let row = rows[marker.index];
        match row {
            SettingRow::LaneKey(lane) => {
                overlay.rebinding = Some(RebindingTarget::Lane(lane));
                continue;
            }
            SettingRow::SystemAction(action) => {
                overlay.rebinding = Some(RebindingTarget::System(action));
                continue;
            }
            _ => {}
        }
        let changed = match windows.single_mut() {
            Ok(mut window) => apply_setting_delta(
                row,
                1.0,
                &mut config,
                &mut mix,
                run.as_deref_mut(),
                Some(window.as_mut()),
                Some(&mut winit),
            ),
            Err(_) => apply_setting_delta(
                row,
                1.0,
                &mut config,
                &mut mix,
                run.as_deref_mut(),
                None,
                Some(&mut winit),
            ),
        };
        if changed && let Err(err) = save_game_config(&config) {
            warn!("failed to save config: {err}");
        }
        return;
    }
}

pub(crate) fn sync_settings_list_scroll(
    overlay: Res<SettingsOverlay>,
    mut sync: ResMut<SettingsScrollSync>,
    mut list_query: Query<
        (Entity, &ComputedNode, &mut ScrollPosition, &Children),
        With<SettingsList>,
    >,
    child_query: Query<&ComputedNode>,
    row_query: Query<&SettingsRowMarker>,
) {
    if sync.selected == overlay.selected
        && sync.category == overlay.category
        && sync.search == overlay.search
    {
        return;
    }

    let category_changed = sync.category != overlay.category;
    let search_changed = sync.search != overlay.search;
    sync.selected = overlay.selected;
    sync.category = overlay.category;
    sync.search.clone_from(&overlay.search);

    let Ok((_, list_computed, mut scroll_position, children)) = list_query.single_mut() else {
        return;
    };

    if category_changed || search_changed {
        scroll_position.y = 0.0;
    }

    let selected = overlay.selected;
    let Some((item_top, item_height)) = child_range_where(
        children,
        &child_query,
        list_computed,
        SPACING_XS,
        |entity| {
            row_query
                .get(entity)
                .is_ok_and(|marker| marker.index == selected)
        },
        |entity| row_query.get(entity).map(|_| SPACING_XS).unwrap_or(0.0),
    ) else {
        return;
    };

    scroll_to_show_range_y(list_computed, &mut scroll_position, item_top, item_height);
}
