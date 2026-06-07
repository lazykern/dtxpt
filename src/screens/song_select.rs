#![allow(clippy::too_many_arguments)]

use std::path::PathBuf;

use bevy::asset::RenderAssetUsages;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use bevy::{asset::Handle, prelude::*};

use crate::app::markers::*;
use crate::app::state::AppState;
use crate::audio::{GameRng, rng_next_usize};
use crate::config::{GameConfig, library_cache_path, save_game_config};
use crate::current_song::{
    CurrentSong, align_library_to_current_song, apply_library_selection,
    enrich_current_song_from_library,
};
use crate::gameplay::SelectedChartPath;
use crate::persistence::ScoreStore;
use crate::ui::fonts::{UiFonts, text_font};
use crate::ui::input::UiKeyRepeat;
use crate::ui::palette::*;
use crate::ui::search_char;
use crate::ui::theme::*;
use crate::ui::widgets::*;
use dtxpt::song_library::{self, pick_chart_index};

#[derive(Resource, Default)]
pub struct SongPreviewImage {
    target_path: Option<PathBuf>,
    displayed_path: Option<PathBuf>,
    image: Option<Handle<Image>>,
    loading: Option<PreviewImageTask>,
    needs_attach: bool,
}

struct PreviewImageTask {
    path: PathBuf,
    task: Task<std::result::Result<Image, String>>,
}

#[derive(Resource, Default)]
pub struct SongSelectUiState {
    search: String,
    selected_entry: usize,
    selected_chart: usize,
    entry_count: usize,
    scanning: bool,
}

pub fn setup_song_select(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    current: Res<CurrentSong>,
    config: Res<GameConfig>,
    mut library: ResMut<song_library::SongLibrary>,
    mut ui_state: ResMut<SongSelectUiState>,
) {
    align_library_to_current_song(&mut library, &current, &config.preferred_difficulty);
    ui_state.entry_count = usize::MAX;
    commands.spawn((
        screen_root(),
        SongSelectScreen,
        children![(
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(SPACING_MD)),
                row_gap: px(SPACING_SM),
                ..default()
            },
            children![
                (
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(px(SPACING_MD), px(SPACING_SM)),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(8.0)),
                        ..default()
                    },
                    BackgroundColor(BG_ELEVATED),
                    BorderColor::all(BORDER_SUBTLE),
                    SongSelectScreen,
                    children![
                        (
                            Text::new("Song Select"),
                            text_font(&fonts, FONT_HEADING),
                            TextColor(TEXT_ACCENT),
                        ),
                        (
                            Text::new("Type search  F3 random  Enter play  Esc menu"),
                            text_font(&fonts, FONT_CAPTION),
                            TextColor(TEXT_MUTED),
                        ),
                    ],
                ),
                (
                    Node {
                        width: percent(100),
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Row,
                        column_gap: px(SPACING_MD),
                        ..default()
                    },
                    children![
                        (
                            Node {
                                flex_basis: percent(58.0),
                                height: percent(100),
                                flex_direction: FlexDirection::Column,
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::all(px(BORDER_RADIUS)),
                                ..default()
                            },
                            BackgroundColor(BG_SECONDARY),
                            BorderColor::all(BORDER_SUBTLE),
                            SongSelectScreen,
                            children![(scroll_list_node(), SongSelectList, SongSelectScreen)],
                        ),
                        (
                            Node {
                                flex_basis: percent(42.0),
                                height: percent(100),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(SPACING_MD),
                                padding: UiRect::all(px(SPACING_MD)),
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::all(px(BORDER_RADIUS)),
                                ..default()
                            },
                            BackgroundColor(BG_SECONDARY),
                            BorderColor::all(BORDER_SUBTLE),
                            SongSelectMeta,
                            SongSelectScreen,
                        ),
                    ],
                ),
                footer_hint_bundle(&fonts, "↑/↓ song  ←/→ difficulty  F1 settings"),
            ],
        )],
    ));
}

fn song_select_list_dirty(
    state: &SongSelectUiState,
    library: &song_library::SongLibrary,
    scanning: bool,
) -> bool {
    state.search != library.search
        || state.selected_entry != library.selected_entry
        || state.entry_count != library.entries.len()
        || state.scanning != scanning
}

fn song_select_chart_dirty(state: &SongSelectUiState, library: &song_library::SongLibrary) -> bool {
    state.selected_chart != library.selected_chart
}

fn list_chart_index(
    library: &song_library::SongLibrary,
    entry_index: usize,
    preferred: &str,
) -> usize {
    let entry = &library.entries[entry_index];
    if entry_index == library.selected_entry {
        library
            .selected_chart
            .min(entry.charts.len().saturating_sub(1))
    } else {
        pick_chart_index(&entry.charts, preferred)
    }
}

fn rebuild_song_list(
    commands: &mut Commands,
    list_entity: Entity,
    fonts: &UiFonts,
    library: &song_library::SongLibrary,
    scan: &song_library::SongLibraryScan,
    scores: &ScoreStore,
    preferred: &str,
) {
    commands.entity(list_entity).despawn_children();

    if scan.scanning {
        commands.entity(list_entity).with_children(|parent| {
            parent.spawn((
                Text::new("Scanning library..."),
                text_font(fonts, FONT_BODY),
                TextColor(WARNING),
            ));
        });
        return;
    }

    if library.entries.is_empty() {
        commands.entity(list_entity).with_children(|parent| {
            parent.spawn((
                Text::new("No songs found.\nPut .dtx folders under chart_root."),
                text_font(fonts, FONT_BODY),
                TextColor(TEXT_SECONDARY),
            ));
        });
        return;
    }

    let visible = library.visible_indices();
    let search_line = format!(
        "Search: {}   Showing {}/{}",
        library.search,
        visible.len(),
        library.entries.len()
    );
    commands.entity(list_entity).with_children(|parent| {
        parent.spawn((
            Text::new(search_line),
            text_font(fonts, FONT_CAPTION),
            TextColor(TEXT_MUTED),
        ));
    });

    if visible.is_empty() {
        commands.entity(list_entity).with_children(|parent| {
            parent.spawn((
                Text::new("No matching songs."),
                text_font(fonts, FONT_BODY),
                TextColor(TEXT_SECONDARY),
            ));
        });
        return;
    }

    let selected_visible = library.selected_visible_index();
    let start = selected_visible.saturating_sub(6);
    let end = (start + 13).min(visible.len());
    for &entry_index in &visible[start..end] {
        let entry = &library.entries[entry_index];
        let selected = entry_index == library.selected_entry;
        let chart_index = list_chart_index(library, entry_index, preferred);
        let chart = &entry.charts[chart_index];
        let artist = entry.artist.as_deref().unwrap_or("");
        let level = chart.level.map(fmt_level).unwrap_or_default();
        let best = if selected {
            scores
                .best_for_path(&chart.path)
                .map(|score| format!("Best {:07}", score.score))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let title_line = format!("{}  [{}] {}", entry.title, chart.label, level);
        let sub_line = format!("{artist} {best}");

        commands.entity(list_entity).with_children(|parent| {
            parent.spawn((
                Button,
                song_card_node(selected),
                BackgroundColor(if selected { CARD_SELECTED } else { CARD_NORMAL }),
                BorderColor::all(if selected {
                    BORDER_FOCUS
                } else {
                    BORDER_SUBTLE
                }),
                SongSelectCard { entry_index },
                children![
                    (
                        Text::new(title_line),
                        text_font(fonts, FONT_BODY),
                        TextColor(if selected {
                            TEXT_PRIMARY
                        } else {
                            TEXT_SECONDARY
                        }),
                    ),
                    (
                        Text::new(sub_line.trim().to_string()),
                        text_font(fonts, FONT_CAPTION),
                        TextColor(TEXT_MUTED),
                    ),
                ],
            ));
        });
    }
}

pub(crate) fn refresh_song_select_ui(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    config: Res<GameConfig>,
    library: Res<song_library::SongLibrary>,
    scan: Res<song_library::SongLibraryScan>,
    scores: Res<ScoreStore>,
    mut ui_state: ResMut<SongSelectUiState>,
    mut preview: ResMut<SongPreviewImage>,
    list_query: Query<Entity, With<SongSelectList>>,
    meta_query: Query<Entity, With<SongSelectMeta>>,
) {
    let preferred = config.preferred_difficulty.as_str();
    let list_dirty = song_select_list_dirty(&ui_state, &library, scan.scanning);
    let chart_dirty = song_select_chart_dirty(&ui_state, &library);
    if !list_dirty && !chart_dirty {
        return;
    }

    if !list_dirty && chart_dirty {
        ui_state.selected_chart = library.selected_chart;
        preview.needs_attach = true;
        if let Ok(list_entity) = list_query.single() {
            rebuild_song_list(
                &mut commands,
                list_entity,
                &fonts,
                &library,
                &scan,
                &scores,
                preferred,
            );
        }
        if let Ok(meta_entity) = meta_query.single() {
            rebuild_meta_panel(&mut commands, meta_entity, &fonts, &library, &scores);
        }
        return;
    }

    let entry_changed = ui_state.selected_entry != library.selected_entry;
    ui_state.search = library.search.clone();
    ui_state.selected_entry = library.selected_entry;
    ui_state.selected_chart = library.selected_chart;
    ui_state.entry_count = library.entries.len();
    ui_state.scanning = scan.scanning;

    let Ok(list_entity) = list_query.single() else {
        return;
    };

    rebuild_song_list(
        &mut commands,
        list_entity,
        &fonts,
        &library,
        &scan,
        &scores,
        preferred,
    );

    let Ok(meta_entity) = meta_query.single() else {
        return;
    };
    if entry_changed {
        preview.needs_attach = true;
    }
    rebuild_meta_panel(&mut commands, meta_entity, &fonts, &library, &scores);
}

fn rebuild_meta_panel(
    commands: &mut Commands,
    meta_entity: Entity,
    fonts: &UiFonts,
    library: &song_library::SongLibrary,
    scores: &ScoreStore,
) {
    commands.entity(meta_entity).despawn_children();
    commands.entity(meta_entity).with_children(|parent| {
        if let Some(entry) = library.current_entry() {
            let chart_index = library
                .selected_chart
                .min(entry.charts.len().saturating_sub(1));
            let chart = &entry.charts[chart_index];
            let artist = entry.artist.as_deref().unwrap_or("Unknown artist");
            let level = chart.level.map(fmt_level_num).unwrap_or_else(|| "?".into());
            let best = scores
                .best_for_path(&chart.path)
                .map(|s| format!("{:07} ({:.2}%)", s.score, s.accuracy))
                .unwrap_or_else(|| "—".into());

            parent.spawn((
                Node {
                    width: px(PREVIEW_PANEL_MAX.x),
                    height: px(PREVIEW_PANEL_MAX.y),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(1.5)),
                    border_radius: BorderRadius::all(px(8.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                },
                BackgroundColor(BG_ELEVATED),
                BorderColor::all(BORDER_SUBTLE),
                SongSelectPreviewFrame,
            ));

            parent.spawn((
                Text::new(&entry.title),
                text_font(fonts, FONT_HEADING),
                TextColor(TEXT_ACCENT),
            ));
            parent.spawn((
                Text::new(artist),
                text_font(fonts, FONT_BODY),
                TextColor(TEXT_SECONDARY),
            ));
            parent.spawn((
                Text::new(format!("[{}] Level {}", chart.label, level)),
                text_font(fonts, FONT_BODY),
                TextColor(TEXT_PRIMARY),
            ));
            parent.spawn((
                Text::new(format!("Best score: {best}")),
                text_font(fonts, FONT_BODY),
                TextColor(SUCCESS),
            ));
            if !entry.box_path.is_empty() {
                parent.spawn((
                    Text::new(format!("Group: {}", entry.box_path.join(" / "))),
                    text_font(fonts, FONT_CAPTION),
                    TextColor(TEXT_MUTED),
                ));
            }
            parent.spawn((
                Text::new(format!(
                    "{} chart(s) in {}",
                    entry.charts.len(),
                    entry.folder.display()
                )),
                text_font(fonts, FONT_CAPTION),
                TextColor(TEXT_MUTED),
            ));
        } else {
            parent.spawn((
                Text::new("Select a song"),
                text_font(fonts, FONT_BODY),
                TextColor(TEXT_SECONDARY),
            ));
        }
    });
}

fn confirm_chart_play(
    library: &song_library::SongLibrary,
    current: &mut CurrentSong,
    config: &mut GameConfig,
    selected: &mut SelectedChartPath,
    next_state: &mut NextState<AppState>,
) {
    if library.current_chart().is_none() {
        return;
    }
    apply_library_selection(current, library);
    current.sync_selected_chart_path(selected);
    current.persist_last_chart(config);
    next_state.set(AppState::LoadingSong);
}

pub(crate) fn song_select_card_interaction(
    mut library: ResMut<song_library::SongLibrary>,
    mut current: ResMut<CurrentSong>,
    mut config: ResMut<GameConfig>,
    mut selected: ResMut<SelectedChartPath>,
    mut next_state: ResMut<NextState<AppState>>,
    cards: Query<(&Interaction, &SongSelectCard), Changed<Interaction>>,
) {
    for (interaction, card) in &cards {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let already_selected = library.selected_entry == card.entry_index;
        library.selected_entry = card.entry_index;
        library.apply_preferred_difficulty(&config.preferred_difficulty);
        if already_selected {
            confirm_chart_play(
                &library,
                &mut current,
                &mut config,
                &mut selected,
                &mut next_state,
            );
        }
    }
}

pub(crate) fn song_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut repeat: Local<UiKeyRepeat>,
    mut library: ResMut<song_library::SongLibrary>,
    mut current: ResMut<CurrentSong>,
    mut config: ResMut<GameConfig>,
    mut selected: ResMut<SelectedChartPath>,
    mut rng: ResMut<GameRng>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut changed = false;
    if keyboard.just_pressed(KeyCode::Backspace) {
        library.search.pop();
        library.normalize_selection(&config.preferred_difficulty);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Delete) {
        library.search.clear();
        library.normalize_selection(&config.preferred_difficulty);
        changed = true;
    }
    for key in keyboard.get_just_pressed() {
        if *key != KeyCode::Space
            && let Some(ch) = search_char(*key)
        {
            library.search.push(ch);
            library.normalize_selection(&config.preferred_difficulty);
            changed = true;
        }
    }
    if keyboard.just_pressed(KeyCode::F3) {
        let random_index = rng_next_usize(&mut rng);
        library.select_random(random_index, &config.preferred_difficulty);
        changed = true;
    }
    if let Some(key) = repeat.update(
        &keyboard,
        &time,
        &[
            KeyCode::ArrowDown,
            KeyCode::ArrowUp,
            KeyCode::ArrowRight,
            KeyCode::ArrowLeft,
        ],
    ) {
        match key {
            KeyCode::ArrowDown => library.select_next(&config.preferred_difficulty),
            KeyCode::ArrowUp => library.select_previous(&config.preferred_difficulty),
            KeyCode::ArrowRight => library.select_next_chart(),
            KeyCode::ArrowLeft => library.select_previous_chart(),
            _ => {}
        }
        if matches!(key, KeyCode::ArrowRight | KeyCode::ArrowLeft)
            && let Some(label) = library.current_chart().map(|chart| chart.label.clone())
        {
            persist_preferred_difficulty(&mut config, &label);
        }
        changed = true;
    }

    if changed {
        let _ = changed;
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        confirm_chart_play(
            &library,
            &mut current,
            &mut config,
            &mut selected,
            &mut next_state,
        );
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

pub(crate) fn sync_current_song_from_library(
    library: Res<song_library::SongLibrary>,
    mut current: ResMut<CurrentSong>,
    mut selected: ResMut<SelectedChartPath>,
    mut last: Local<Option<(usize, usize, String)>>,
) {
    let key = (
        library.selected_entry,
        library.selected_chart,
        library.search.clone(),
    );
    if *last == Some(key.clone()) {
        return;
    }
    *last = Some(key);
    apply_library_selection(&mut current, &library);
    current.sync_selected_chart_path(&mut selected);
}

fn load_preview_image(path: PathBuf) -> std::result::Result<Image, String> {
    let bytes =
        std::fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let dynamic = image::load_from_memory(&bytes)
        .map_err(|err| format!("failed to decode {}: {err}", path.display()))?;
    Ok(Image::from_dynamic(
        dynamic,
        true,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

pub(crate) fn update_song_preview_image(
    library: Res<song_library::SongLibrary>,
    mut commands: Commands,
    mut preview: ResMut<SongPreviewImage>,
    mut images: ResMut<Assets<Image>>,
    frames: Query<Entity, With<SongSelectPreviewFrame>>,
) {
    let next_path = library
        .current_entry()
        .and_then(|entry| entry.preview_image.clone());

    if preview.target_path != next_path {
        preview.target_path = next_path.clone();
        preview.loading = None;
        preview.needs_attach = true;

        match next_path {
            Some(path) if preview.displayed_path.as_ref() != Some(&path) => {
                preview.loading = Some(PreviewImageTask {
                    path: path.clone(),
                    task: AsyncComputeTaskPool::get()
                        .spawn(async move { load_preview_image(path) }),
                });
            }
            Some(_) => {}
            None => {
                preview.displayed_path = None;
                preview.image = None;
            }
        }
    }

    if let Some(loading) = preview.loading.as_mut()
        && let Some(result) = check_ready(&mut loading.task)
    {
        let loaded_path = loading.path.clone();
        preview.loading = None;
        if preview.target_path.as_ref() == Some(&loaded_path) {
            match result {
                Ok(image) => {
                    preview.image = Some(images.add(image));
                    preview.displayed_path = Some(loaded_path);
                    preview.needs_attach = true;
                }
                Err(err) => {
                    warn!("failed to load preview image: {err}");
                    preview.image = None;
                    preview.displayed_path = None;
                    preview.needs_attach = true;
                }
            }
        }
    }

    if !preview.needs_attach {
        return;
    }

    let Ok(frame_entity) = frames.single() else {
        return;
    };

    sync_preview_frame(&mut commands, frame_entity, &preview);
    preview.needs_attach = false;
}

fn sync_preview_frame(commands: &mut Commands, frame_entity: Entity, preview: &SongPreviewImage) {
    commands.entity(frame_entity).despawn_children();

    if let Some(handle) = preview.image.clone() {
        let image_entity = commands
            .spawn((
                ImageNode::new(handle),
                Node {
                    width: px(PREVIEW_PANEL_MAX.x - 8.0),
                    height: px(PREVIEW_PANEL_MAX.y - 8.0),
                    ..default()
                },
                SongSelectPreviewImage,
                SongSelectScreen,
            ))
            .id();
        commands.entity(frame_entity).add_child(image_entity);
    }
}

pub fn stop_song_preview_image(mut preview: ResMut<SongPreviewImage>) {
    preview.target_path = None;
    preview.displayed_path = None;
    preview.image = None;
    preview.loading = None;
    preview.needs_attach = false;
}

pub(crate) fn persist_current_song_on_exit_song_select(
    current: Res<CurrentSong>,
    mut config: ResMut<GameConfig>,
) {
    current.persist_last_chart(&mut config);
}

pub(crate) fn poll_song_library_scan(
    mut library: ResMut<song_library::SongLibrary>,
    mut scan: ResMut<song_library::SongLibraryScan>,
    mut current: ResMut<CurrentSong>,
    mut selected: ResMut<SelectedChartPath>,
    config: Res<GameConfig>,
) {
    let Some(updated) = scan.poll() else {
        return;
    };

    *library = updated;
    library.normalize_selection(&config.preferred_difficulty);
    enrich_current_song_from_library(&mut current, &mut library);
    current.sync_selected_chart_path(&mut selected);
    if let Err(err) =
        song_library::save_library_cache(&library_cache_path(), &config.chart_root, &library)
    {
        warn!("failed to save library cache: {err}");
    }
    info!(
        "song library scan complete: {} entries",
        library.entries.len()
    );
}

fn persist_preferred_difficulty(config: &mut GameConfig, label: &str) {
    if label.is_empty() || config.preferred_difficulty == label {
        return;
    }
    config.preferred_difficulty = label.to_string();
    if let Err(err) = save_game_config(config) {
        warn!("failed to save preferred difficulty: {err}");
    }
}

fn fmt_level(level: f32) -> String {
    if level.fract() == 0.0 {
        format!("Lv{:.0}", level)
    } else {
        format!("Lv{:.1}", level)
    }
}

fn fmt_level_num(level: f32) -> String {
    if level.fract() == 0.0 {
        format!("{:.0}", level)
    } else {
        format!("{:.1}", level)
    }
}
