use bevy::prelude::*;

use super::fonts::{UiFonts, text_font};
use super::palette::*;
use super::theme::*;

#[derive(Component, Clone, Copy)]
pub struct UiButton {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

impl Default for UiButton {
    fn default() -> Self {
        Self {
            normal: BUTTON_NORMAL,
            hovered: BUTTON_HOVER,
            pressed: BUTTON_PRESSED,
        }
    }
}

#[derive(Component)]
pub struct UiPanel;

#[derive(Component)]
pub struct UiLabel;

pub fn screen_root() -> Node {
    Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

pub fn centered_column(gap: f32) -> Node {
    Node {
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: px(gap),
        padding: UiRect::all(px(PANEL_PADDING)),
        ..default()
    }
}

pub fn panel_node(width: Val, height: Val) -> Node {
    Node {
        width,
        height,
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(PANEL_PADDING)),
        border: UiRect::all(px(1.5)),
        border_radius: BorderRadius::all(px(BORDER_RADIUS)),
        ..default()
    }
}

pub fn panel_bundle(fonts: &UiFonts, title: &str, width: Val, height: Val) -> impl Bundle {
    (
        panel_node(width, height),
        BackgroundColor(BG_SECONDARY),
        BorderColor::all(BORDER_SUBTLE),
        UiPanel,
        children![(
            Text::new(title),
            text_font(fonts, FONT_HEADING),
            TextColor(TEXT_ACCENT),
            UiLabel,
        )],
    )
}

pub fn spawn_panel(
    commands: &mut Commands,
    fonts: &UiFonts,
    title: &str,
    width: Val,
    height: Val,
) -> Entity {
    commands
        .spawn(panel_bundle(fonts, title, width, height))
        .id()
}

pub fn button_bundle(fonts: &UiFonts, label: &str, width: f32, height: f32) -> impl Bundle {
    (
        Button,
        UiButton::default(),
        Node {
            width: px(width),
            height: px(height),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(1.5)),
            border_radius: BorderRadius::all(px(BORDER_RADIUS)),
            ..default()
        },
        BackgroundColor(BUTTON_NORMAL),
        BorderColor::all(BORDER_SUBTLE),
        children![(
            Text::new(label),
            text_font(fonts, FONT_BODY),
            TextColor(TEXT_PRIMARY),
        )],
    )
}

pub fn spawn_button(
    commands: &mut Commands,
    fonts: &UiFonts,
    label: &str,
    width: f32,
    height: f32,
) -> Entity {
    commands
        .spawn(button_bundle(fonts, label, width, height))
        .id()
}

pub fn caption_bundle(fonts: &UiFonts, text: impl Into<String>, color: Color) -> impl Bundle {
    (
        Text::new(text.into()),
        text_font(fonts, FONT_CAPTION),
        TextColor(color),
        UiLabel,
    )
}

pub fn spawn_caption(commands: &mut Commands, fonts: &UiFonts, text: &str, color: Color) -> Entity {
    commands.spawn(caption_bundle(fonts, text, color)).id()
}

pub fn heading_bundle(fonts: &UiFonts, text: impl Into<String>) -> impl Bundle {
    (
        Text::new(text.into()),
        text_font(fonts, FONT_TITLE),
        TextColor(TEXT_ACCENT),
        UiLabel,
    )
}

pub fn spawn_heading(commands: &mut Commands, fonts: &UiFonts, text: &str) -> Entity {
    commands.spawn(heading_bundle(fonts, text)).id()
}

pub fn body_text_bundle(fonts: &UiFonts, text: impl Into<String>, color: Color) -> impl Bundle {
    (
        Text::new(text.into()),
        text_font(fonts, FONT_BODY),
        TextColor(color),
        UiLabel,
    )
}

pub fn spawn_body_text(
    commands: &mut Commands,
    fonts: &UiFonts,
    text: &str,
    color: Color,
) -> Entity {
    commands.spawn(body_text_bundle(fonts, text, color)).id()
}

pub fn footer_bar() -> Node {
    Node {
        width: percent(100),
        padding: UiRect::axes(px(SPACING_MD), px(SPACING_SM)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::top(px(1.0)),
        ..default()
    }
}

pub fn footer_hint_bundle(fonts: &UiFonts, text: &str) -> impl Bundle {
    (
        footer_bar(),
        BorderColor::all(BORDER_SUBTLE),
        children![(
            Text::new(text),
            text_font(fonts, FONT_CAPTION),
            TextColor(TEXT_MUTED),
        )],
    )
}

pub fn spawn_footer_hint(commands: &mut Commands, fonts: &UiFonts, text: &str) -> Entity {
    commands.spawn(footer_hint_bundle(fonts, text)).id()
}

pub fn overlay_backdrop_bundle() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(BG_OVERLAY),
        ZIndex(100),
    )
}

pub fn spawn_overlay_backdrop(commands: &mut Commands) -> Entity {
    commands.spawn(overlay_backdrop_bundle()).id()
}

pub fn search_bar_bundle(
    fonts: &UiFonts,
    value: &str,
    placeholder: &str,
    width: f32,
) -> impl Bundle {
    let text = if value.trim().is_empty() {
        placeholder.to_string()
    } else {
        format!("Search: {value}")
    };
    (
        Node {
            width: px(width),
            height: px(42.0),
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(SPACING_MD)),
            border: UiRect::all(px(1.0)),
            border_radius: BorderRadius::all(px(21.0)),
            ..default()
        },
        BackgroundColor(BG_ELEVATED),
        BorderColor::all(BORDER_SUBTLE),
        children![(
            Text::new(text),
            text_font(fonts, FONT_CAPTION),
            TextColor(if value.trim().is_empty() {
                TEXT_MUTED
            } else {
                TEXT_PRIMARY
            }),
        )],
    )
}

pub fn stat_row_bundle(fonts: &UiFonts, label: &str, value: &str) -> impl Bundle {
    (
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::vertical(px(SPACING_XS)),
            ..default()
        },
        children![
            (
                Text::new(label),
                text_font(fonts, FONT_BODY),
                TextColor(TEXT_SECONDARY),
            ),
            (
                Text::new(value),
                text_font(fonts, FONT_BODY),
                TextColor(TEXT_PRIMARY),
            ),
        ],
    )
}

pub fn spawn_stat_row(
    commands: &mut Commands,
    fonts: &UiFonts,
    label: &str,
    value: &str,
) -> Entity {
    commands.spawn(stat_row_bundle(fonts, label, value)).id()
}

pub fn progress_bar_bundle(
    width: f32,
    height: f32,
    fill_ratio: f32,
    fill_color: Color,
) -> impl Bundle {
    (
        Node {
            width: px(width),
            height: px(height),
            border_radius: BorderRadius::all(px(height / 2.0)),
            ..default()
        },
        BackgroundColor(BG_ELEVATED),
        children![(
            Node {
                width: percent(fill_ratio.clamp(0.0, 1.0) * 100.0),
                height: percent(100),
                border_radius: BorderRadius::all(px(height / 2.0)),
                ..default()
            },
            BackgroundColor(fill_color),
        )],
    )
}

pub fn spawn_progress_bar(
    commands: &mut Commands,
    width: f32,
    height: f32,
    fill_ratio: f32,
    fill_color: Color,
) -> Entity {
    commands
        .spawn(progress_bar_bundle(width, height, fill_ratio, fill_color))
        .id()
}

pub fn slider_visual_bundle(width: f32, ratio: f32) -> impl Bundle {
    (
        Node {
            width: px(width),
            height: px(8.0),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BackgroundColor(BG_ELEVATED),
        children![(
            Node {
                width: percent(ratio.clamp(0.0, 1.0) * 100.0),
                height: percent(100),
                border_radius: BorderRadius::all(px(4.0)),
                ..default()
            },
            BackgroundColor(ACCENT),
        )],
    )
}

pub fn spawn_slider_visual(commands: &mut Commands, width: f32, ratio: f32) -> Entity {
    commands.spawn(slider_visual_bundle(width, ratio)).id()
}

pub fn toggle_visual_bundle(enabled: bool) -> impl Bundle {
    let (bg, knob_x) = if enabled {
        (ACCENT, 18.0)
    } else {
        (BG_ELEVATED, 2.0)
    };
    (
        Node {
            width: px(40.0),
            height: px(22.0),
            border_radius: BorderRadius::all(px(11.0)),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(2.0)),
            ..default()
        },
        BackgroundColor(bg),
        children![(
            Node {
                width: px(18.0),
                height: px(18.0),
                margin: UiRect::left(px(knob_x)),
                border_radius: BorderRadius::all(px(9.0)),
                ..default()
            },
            BackgroundColor(TEXT_PRIMARY),
        )],
    )
}

pub fn spawn_toggle_visual(commands: &mut Commands, enabled: bool) -> Entity {
    commands.spawn(toggle_visual_bundle(enabled)).id()
}

pub fn song_card_node(selected: bool) -> Node {
    Node {
        width: percent(100),
        min_height: px(52.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        padding: UiRect::axes(px(SPACING_MD), px(SPACING_SM)),
        margin: UiRect::bottom(px(SPACING_XS)),
        border_radius: BorderRadius::all(px(6.0)),
        border: UiRect::all(px(if selected { 2.0 } else { 1.0 })),
        ..default()
    }
}

pub fn scroll_list_node() -> Node {
    Node {
        flex_grow: 1.0,
        flex_direction: FlexDirection::Column,
        overflow: Overflow::scroll_y(),
        row_gap: px(SPACING_XS),
        padding: UiRect::all(px(SPACING_SM)),
        ..default()
    }
}
