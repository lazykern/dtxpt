#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use dtxpt::chart::Chart;
use dtxpt::input::SystemAction;

use crate::app::markers::{
    GameplayEntity, MetronomeLineVisual, NoteVisual, PauseAction, PauseActionButton,
    PauseOverlayScreen,
};
use crate::app::state::{AppState, OverlayState, PauseState, is_paused, overlay_closed};
use crate::audio::{
    self, ActiveSounds, AudioFrame, AudioMix, BgmInstance, BoundInput, MetronomeActive, SoundBank,
};
use crate::gameplay::clock::ChartClock;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;
use crate::ui::fonts::{UiFonts, text_font};
use crate::ui::input::UiKeyRepeat;
use crate::ui::palette::*;
use crate::ui::theme::*;
use crate::ui::widgets::{UiButton, button_bundle, screen_root};

const PAUSE_CHOICES: [PauseAction; 3] = [
    PauseAction::Resume,
    PauseAction::Retry,
    PauseAction::SongSelect,
];

#[derive(Resource, Default)]
pub struct PauseUiState {
    pub selected: usize,
}

#[derive(SystemParam)]
pub(crate) struct PauseMenuActions<'w, 's> {
    next_pause: ResMut<'w, NextState<PauseState>>,
    next_state: ResMut<'w, NextState<AppState>>,
    chart: ResMut<'w, Chart>,
    run: ResMut<'w, RunState>,
    clock: ResMut<'w, ChartClock>,
    layout: Res<'w, PlayfieldLayout>,
    commands: Commands<'w, 's>,
    audio_instances: ResMut<'w, Assets<AudioInstance>>,
    bgm_instance: Option<Res<'w, BgmInstance>>,
    active: ResMut<'w, ActiveSounds>,
    metronome_active: ResMut<'w, MetronomeActive>,
    sound_bank: Res<'w, SoundBank>,
    mix: Res<'w, AudioMix>,
    audio: Res<'w, Audio>,
    frame: Res<'w, AudioFrame>,
    visuals: ParamSet<
        'w,
        's,
        (
            Query<'w, 's, Entity, With<NoteVisual>>,
            Query<'w, 's, Entity, With<MetronomeLineVisual>>,
        ),
    >,
}

impl PauseMenuActions<'_, '_> {
    fn apply(&mut self, action: PauseAction) {
        match action {
            PauseAction::Resume => {
                self.next_pause.set(PauseState::Running);
                let bgm_handle = self
                    .bgm_instance
                    .as_ref()
                    .map(|bgm| bgm.handle.clone())
                    .or_else(|| {
                        audio::start_bgm_at_chart_time(
                            &mut self.commands,
                            &mut self.chart,
                            self.clock.audio_elapsed,
                            self.run.song_playback_rate,
                            self.frame.0,
                            &self.sound_bank,
                            &self.mix,
                            &self.audio,
                            &mut self.audio_instances,
                        )
                    });
                audio::set_playback_paused(
                    false,
                    bgm_handle.as_ref(),
                    &mut self.active,
                    &mut self.metronome_active,
                    &mut self.audio_instances,
                );
                info!("playback resumed from pause menu");
            }
            PauseAction::Retry => {
                audio::restart_playback(
                    &mut self.chart,
                    &mut self.run,
                    &mut self.clock,
                    &self.layout,
                    &mut self.commands,
                    &mut self.audio_instances,
                    self.bgm_instance.take(),
                    &mut self.active,
                    &mut self.metronome_active,
                    &mut self.visuals,
                );
                self.next_pause.set(PauseState::Running);
                let bgm_handle = audio::start_bgm_at_chart_time(
                    &mut self.commands,
                    &mut self.chart,
                    self.clock.audio_elapsed,
                    self.run.song_playback_rate,
                    self.frame.0,
                    &self.sound_bank,
                    &self.mix,
                    &self.audio,
                    &mut self.audio_instances,
                );
                audio::set_playback_paused(
                    false,
                    bgm_handle.as_ref(),
                    &mut self.active,
                    &mut self.metronome_active,
                    &mut self.audio_instances,
                );
                info!("restart from pause menu");
            }
            PauseAction::SongSelect => {
                self.next_state.set(AppState::SongSelect);
            }
        }
    }
}

pub(crate) fn toggle_playback_pause(
    input: BoundInput,
    overlay_state: Res<State<OverlayState>>,
    pause_state: Res<State<PauseState>>,
    mut next_pause: ResMut<NextState<PauseState>>,
    mut chart: ResMut<Chart>,
    run: Res<RunState>,
    clock: Res<ChartClock>,
    mut commands: Commands,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    bgm_instance: Option<Res<BgmInstance>>,
    mut active: ResMut<ActiveSounds>,
    mut metronome_active: ResMut<MetronomeActive>,
    sound_bank: Res<SoundBank>,
    mix: Res<AudioMix>,
    audio: Res<Audio>,
    frame: Res<AudioFrame>,
) {
    if !overlay_closed(overlay_state) {
        return;
    }
    if !input.action_just_pressed(SystemAction::PauseToggle) {
        return;
    }

    let resuming = is_paused(pause_state.get());
    let paused = !resuming;
    next_pause.set(if paused {
        PauseState::Paused
    } else {
        PauseState::Running
    });

    let mut bgm_handle = bgm_instance.as_ref().map(|bgm| bgm.handle.clone());
    if resuming && !paused && bgm_handle.is_none() {
        bgm_handle = audio::start_bgm_at_chart_time(
            &mut commands,
            &mut chart,
            clock.audio_elapsed,
            run.song_playback_rate,
            frame.0,
            &sound_bank,
            &mix,
            &audio,
            &mut audio_instances,
        );
    }
    audio::set_playback_paused(
        paused,
        bgm_handle.as_ref(),
        &mut active,
        &mut metronome_active,
        &mut audio_instances,
    );
    info!("playback {}", if paused { "paused" } else { "resumed" });
}

pub(crate) fn update_pause_overlay(
    mut commands: Commands,
    pause_state: Res<State<PauseState>>,
    mut ui_state: ResMut<PauseUiState>,
    fonts: Res<UiFonts>,
    overlays: Query<Entity, With<PauseOverlayScreen>>,
) {
    if is_paused(pause_state.get()) {
        if !overlays.is_empty() {
            return;
        }
        ui_state.selected = 0;
        commands.spawn((
            screen_root(),
            ZIndex(150),
            PauseOverlayScreen,
            GameplayEntity,
            children![
                (
                    Node {
                        width: percent(100),
                        height: percent(100),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(BG_OVERLAY),
                ),
                (
                    Node {
                        width: px(460.0),
                        margin: UiRect::all(auto()),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(SPACING_MD),
                        padding: UiRect::all(px(SPACING_XL)),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(BORDER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(BG_SECONDARY),
                    BorderColor::all(BORDER_SUBTLE),
                    children![
                        (
                            Text::new("Paused"),
                            text_font(&fonts, FONT_HEADING),
                            TextColor(TEXT_ACCENT),
                        ),
                        (
                            button_bundle(&fonts, "Resume (Esc)", 280.0, 48.0),
                            PauseActionButton(PauseAction::Resume),
                        ),
                        (
                            button_bundle(&fonts, "Retry (R)", 280.0, 48.0),
                            PauseActionButton(PauseAction::Retry),
                        ),
                        (
                            button_bundle(&fonts, "Song Select", 280.0, 48.0),
                            PauseActionButton(PauseAction::SongSelect),
                        ),
                        (
                            Text::new("↑/↓ select  Enter activate  Esc resume  F1 settings"),
                            text_font(&fonts, FONT_CAPTION),
                            TextColor(TEXT_MUTED),
                        ),
                    ],
                ),
            ],
        ));
        return;
    }

    for entity in &overlays {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut repeat: Local<UiKeyRepeat>,
    overlay_state: Res<State<OverlayState>>,
    pause_state: Res<State<PauseState>>,
    mut ui_state: ResMut<PauseUiState>,
    mut actions: PauseMenuActions,
    buttons: Query<(&Interaction, &PauseActionButton), Changed<Interaction>>,
) {
    if !overlay_closed(overlay_state) || !is_paused(pause_state.get()) {
        return;
    }

    for (interaction, action) in &buttons {
        if *interaction == Interaction::Pressed {
            actions.apply(action.0);
        }
    }

    if let Some(key) = repeat.update(&keyboard, &time, &[KeyCode::ArrowDown, KeyCode::ArrowUp]) {
        match key {
            KeyCode::ArrowDown => {
                ui_state.selected = (ui_state.selected + 1) % PAUSE_CHOICES.len();
            }
            KeyCode::ArrowUp => {
                ui_state.selected = if ui_state.selected == 0 {
                    PAUSE_CHOICES.len() - 1
                } else {
                    ui_state.selected - 1
                };
            }
            _ => {}
        }
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        actions.apply(PauseAction::Retry);
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        actions.apply(PAUSE_CHOICES[ui_state.selected]);
    }
}

pub(crate) fn sync_pause_focus(
    pause_state: Res<State<PauseState>>,
    ui_state: Res<PauseUiState>,
    mut buttons: Query<(
        &PauseActionButton,
        &Interaction,
        &UiButton,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    if !is_paused(pause_state.get()) {
        return;
    }

    for (action, interaction, style, mut bg, mut border) in &mut buttons {
        let focused = PAUSE_CHOICES[ui_state.selected] == action.0;
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
