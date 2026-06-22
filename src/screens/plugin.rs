use bevy::prelude::*;

use crate::app::markers::{LoadingScreen, MainMenuScreen, ResultScreen, SongSelectScreen};
use crate::app::state::{AppState, OverlayState};
use crate::audio::{stop_menu_music, update_menu_music};
use crate::overlays::settings::{setup_global, track_window_focus};
use crate::screens::common::cleanup_screen;
use crate::screens::loading::{
    ChartLoad, poll_chart_load, reset_chart_load, setup_loading_screen, start_chart_load,
};
use crate::screens::main_menu::{
    MainMenuUiState, main_menu_input, setup_main_menu, sync_main_menu_focus,
    sync_main_menu_song_ticker,
};
use crate::screens::menu_background::{setup_menu_background, sync_menu_background_visibility};
use crate::screens::result::{result_input, setup_result};
use crate::screens::stage_clear::{setup_stage_clear, setup_stage_failed, stage_clear_auto_advance};
use crate::screens::song_select::{
    SongSelectUiState, persist_current_song_on_exit_song_select, poll_song_library_scan,
    refresh_song_select_ui, setup_song_select, song_select_card_interaction, song_select_input,
    stop_song_preview_image, sync_current_song_from_library, update_song_preview_image,
};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChartLoad>()
            .init_resource::<MainMenuUiState>()
            .init_resource::<SongSelectUiState>()
            .add_systems(Startup, (setup_global, setup_menu_background).chain())
            .add_systems(
                Update,
                (track_window_focus, sync_menu_background_visibility, poll_song_library_scan),
            )
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                (
                    main_menu_input,
                    sync_main_menu_focus.after(main_menu_input),
                    sync_main_menu_song_ticker,
                )
                    .run_if(in_state(AppState::MainMenu))
                    .run_if(in_state(OverlayState::None)),
            )
            .add_systems(OnExit(AppState::MainMenu), cleanup_screen::<MainMenuScreen>)
            .add_systems(OnEnter(AppState::SongSelect), setup_song_select)
            .add_systems(
                Update,
                (
                    song_select_input,
                    song_select_card_interaction.after(song_select_input),
                    sync_current_song_from_library.after(song_select_card_interaction),
                    refresh_song_select_ui.after(sync_current_song_from_library),
                )
                    .run_if(in_state(AppState::SongSelect))
                    .run_if(in_state(OverlayState::None)),
            )
            .add_systems(
                Update,
                (
                    update_menu_music,
                    update_song_preview_image.after(refresh_song_select_ui),
                )
                    .run_if(in_state(AppState::MainMenu).or(in_state(AppState::SongSelect))),
            )
            .add_systems(
                OnExit(AppState::SongSelect),
                (
                    cleanup_screen::<SongSelectScreen>,
                    persist_current_song_on_exit_song_select,
                    stop_song_preview_image,
                ),
            )
            .add_systems(
                OnEnter(AppState::LoadingSong),
                (stop_menu_music, setup_loading_screen, start_chart_load).chain(),
            )
            .add_systems(
                Update,
                poll_chart_load.run_if(in_state(AppState::LoadingSong)),
            )
            .add_systems(
                OnExit(AppState::LoadingSong),
                (cleanup_screen::<LoadingScreen>, reset_chart_load),
            )
            .add_systems(OnEnter(AppState::Result), setup_result)
            .add_systems(
                Update,
                result_input
                    .run_if(in_state(AppState::Result))
                    .run_if(in_state(OverlayState::None)),
            )
            .add_systems(OnExit(AppState::Result), cleanup_screen::<ResultScreen>)
            .add_systems(OnEnter(AppState::StageClear), setup_stage_clear)
            .add_systems(OnEnter(AppState::StageFailed), setup_stage_failed)
            .add_systems(
                Update,
                stage_clear_auto_advance
                    .run_if(in_state(AppState::StageClear).or(in_state(AppState::StageFailed))),
            );
    }
}
