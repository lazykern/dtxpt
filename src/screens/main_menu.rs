#![allow(clippy::too_many_arguments)]

use bevy::app::AppExit;
use bevy::prelude::*;

use crate::app::markers::{MainMenuAction, MainMenuChoice, MainMenuScreen};
use crate::app::state::{AppState, OverlayState};
use crate::current_song::CurrentSong;
use crate::ui::animation::screen_fade_in;
use crate::ui::fonts::{UiFonts, text_font};
use crate::ui::input::UiKeyRepeat;
use crate::ui::palette::*;
use crate::ui::theme::*;
use crate::ui::widgets::*;

#[derive(Resource, Default)]
pub struct MainMenuUiState {
    selected: usize,
}

#[derive(Component)]
pub(crate) struct MainMenuSongTicker;

const MAIN_MENU_CHOICES: [MainMenuChoice; 3] = [
    MainMenuChoice::SongSelect,
    MainMenuChoice::Settings,
    MainMenuChoice::Quit,
];

pub fn setup_main_menu(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    current: Res<CurrentSong>,
    mut ui_state: ResMut<MainMenuUiState>,
) {
    ui_state.selected = 0;
    let ticker_line = current.display_line().unwrap_or_default();
    commands.spawn((
        screen_root(),
        MainMenuScreen,
        screen_fade_in(0.35),
        children![(
            centered_column(SPACING_LG),
            children![
                (
                    Text::new("dtxpt"),
                    text_font(&fonts, FONT_TITLE),
                    TextColor(TEXT_ACCENT),
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(SPACING_MD),
                        padding: UiRect::all(px(SPACING_LG)),
                        border: UiRect::all(px(1.5)),
                        border_radius: BorderRadius::all(px(BORDER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(BG_SECONDARY),
                    BorderColor::all(BORDER_SUBTLE),
                    children![
                        (
                            button_bundle(&fonts, "Song Select", 280.0, 56.0),
                            MainMenuAction(MainMenuChoice::SongSelect),
                        ),
                        (
                            button_bundle(&fonts, "Settings", 280.0, 48.0),
                            MainMenuAction(MainMenuChoice::Settings),
                        ),
                        (
                            button_bundle(&fonts, "Quit", 280.0, 48.0),
                            MainMenuAction(MainMenuChoice::Quit),
                        ),
                    ],
                ),
                (
                    Text::new(ticker_line),
                    text_font(&fonts, FONT_CAPTION),
                    TextColor(TEXT_MUTED),
                    MainMenuSongTicker,
                ),
                (
                    Text::new("↑/↓ select  Enter activate  Esc quit"),
                    text_font(&fonts, FONT_CAPTION),
                    TextColor(TEXT_MUTED),
                ),
            ],
        )],
    ));
}

pub(crate) fn main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut repeat: Local<UiKeyRepeat>,
    mut ui_state: ResMut<MainMenuUiState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_overlay: ResMut<NextState<OverlayState>>,
    mut exit: MessageWriter<AppExit>,
    buttons: Query<(&Interaction, &MainMenuAction), Changed<Interaction>>,
) {
    for (interaction, action) in &buttons {
        if *interaction == Interaction::Pressed {
            match action.0 {
                MainMenuChoice::SongSelect => next_state.set(AppState::SongSelect),
                MainMenuChoice::Settings => next_overlay.set(OverlayState::Settings),
                MainMenuChoice::Quit => {
                    exit.write(AppExit::Success);
                }
            }
        }
    }

    if let Some(key) = repeat.update(&keyboard, &time, &[KeyCode::ArrowDown, KeyCode::ArrowUp]) {
        match key {
            KeyCode::ArrowDown => {
                ui_state.selected = (ui_state.selected + 1) % MAIN_MENU_CHOICES.len()
            }
            KeyCode::ArrowUp => {
                ui_state.selected = if ui_state.selected == 0 {
                    MAIN_MENU_CHOICES.len() - 1
                } else {
                    ui_state.selected - 1
                };
            }
            _ => {}
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        match MAIN_MENU_CHOICES[ui_state.selected] {
            MainMenuChoice::SongSelect => next_state.set(AppState::SongSelect),
            MainMenuChoice::Settings => next_overlay.set(OverlayState::Settings),
            MainMenuChoice::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

pub(crate) fn sync_main_menu_song_ticker(
    current: Res<CurrentSong>,
    mut ticker: Query<&mut Text, With<MainMenuSongTicker>>,
) {
    if !current.is_changed() {
        return;
    }
    let line = current.display_line().unwrap_or_default();
    for mut text in &mut ticker {
        text.0 = line.clone();
    }
}

pub(crate) fn sync_main_menu_focus(
    ui_state: Res<MainMenuUiState>,
    mut buttons: Query<(
        &MainMenuAction,
        &Interaction,
        &UiButton,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    for (action, interaction, style, mut bg, mut border) in &mut buttons {
        let focused = MAIN_MENU_CHOICES[ui_state.selected] == action.0;
        if focused {
            bg.0 = CARD_SELECTED;
            *border = BorderColor::all(BORDER_FOCUS);
        } else if *interaction == Interaction::Hovered {
            bg.0 = style.hovered;
            *border = BorderColor::all(BORDER_FOCUS);
        } else {
            bg.0 = style.normal;
            *border = BorderColor::all(BORDER_SUBTLE);
        }
    }
}
