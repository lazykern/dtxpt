#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_kira_audio::{AudioSource, prelude::*};

use dtxpt::chart::dtx::channels::is_dtx_stick_se_channel;
use dtxpt::chart::{Chart, ChartTiming, ScheduledAudioKind};
use dtxpt::input::InputBindings;
use dtxpt::input::lanes::LANES;

use crate::app::markers::*;
use crate::audio::*;
use crate::gameplay::clock::*;
use crate::gameplay::hud::HudDisplayCache;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::rendering::keyboard_viz;
use crate::gameplay::rendering::playfield_viz::lane_receptor_color;
use crate::gameplay::run::*;
use crate::ui::fonts::{UiFonts, text_font};
use crate::ui::palette::*;
use crate::ui::theme::{
    FONT_CAPTION, FONT_HEADING, FONT_HUD, REF_RECEPTOR_H, SPACING_MD, SPACING_SM,
};
use crate::ui::widgets::screen_root;

use crate::gameplay::gauge::spawn_gauge_bar;
use crate::gameplay::metronome::make_metronome_click;
use crate::gameplay::rendering::notes::PlayfieldVisualStreams;

pub fn setup_gameplay(
    mut commands: Commands,
    chart: Res<Chart>,
    run: Res<RunState>,
    bindings: Res<InputBindings>,
    clock: Res<ChartClock>,
    asset_server: Res<AssetServer>,
    fonts: Res<UiFonts>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut layout: ResMut<PlayfieldLayout>,
    _audio: Res<Audio>,
) {
    if let Ok(mut window) = windows.single_mut() {
        *layout = PlayfieldLayout::from_window(&window);
        window.focused = true;
    }
    commands.insert_resource(HudDisplayCache::default());

    let auto_se_count = chart
        .scheduled_audio
        .iter()
        .filter(|e| matches!(e.kind, ScheduledAudioKind::AutoSe { .. }))
        .count();
    info!(
        "dtxpt POC booted: {} notes, {} beats, {} auto SE, {} bpm",
        chart.notes.len(),
        chart.metronome_beats.len(),
        auto_se_count,
        chart.bpm
    );

    let downbeat = asset_server.add(AudioSource {
        sound: make_metronome_click(1_200.0, 35.0, 0.45),
    });
    let beat = asset_server.add(AudioSource {
        sound: make_metronome_click(900.0, 28.0, 0.28),
    });
    commands.insert_resource(MetronomeSounds { downbeat, beat });

    // Load sound bank: only note-referenced WAVs at startup.
    // BGM/stem files referenced only by scheduled audio are loaded deferred during warmup.
    let immediate_ids = collect_immediate_wav_ids(&chart);
    let sound_bank = build_sound_bank_for_ids(&chart, &asset_server, &immediate_ids, "immediate");
    commands.insert_resource(sound_bank);

    let deferred_ids = collect_deferred_wav_ids(&chart);
    if !deferred_ids.is_empty() {
        let entries = build_deferred_entries(&chart, &deferred_ids);
        info!(
            "deferred {} WAV ids for BGM/stem, spawning background decode",
            deferred_ids.len(),
        );
        let (rx, pending) = crate::audio::decode_pool::spawn_bounded_decode(entries);
        commands.insert_resource(BackgroundDecodeReceiver {
            rx: std::sync::Mutex::new(rx),
            pending,
        });
    }

    if let Some(bgm_time) = chart
        .scheduled_audio
        .iter()
        .find(|e| matches!(e.kind, ScheduledAudioKind::Bgm))
        .map(|e| e.time)
    {
        info!("BGM queued at chart time {:.3}s", bgm_time);
    }
    if auto_se_count != 0 {
        let mut muting = 0;
        let mut non_muting = 0;
        for event in &chart.scheduled_audio {
            if let ScheduledAudioKind::AutoSe { channel } = event.kind {
                if is_dtx_stick_se_channel(channel) {
                    muting += 1;
                } else {
                    non_muting += 1;
                }
            }
        }
        info!(
            "queued {} auto SE events ({} muting, {} non-muting)",
            auto_se_count, muting, non_muting
        );
    }

    // Backboard.
    commands.spawn((
        Sprite::from_color(Color::srgb(0.03, 0.035, 0.05), layout.backboard_size),
        Transform::from_xyz(0.0, layout.backboard_center_y, -5.0),
        PlayfieldBackboard,
        GameplayEntity,
    ));

    // Lanes + labels.
    for (lane, spec) in LANES.iter().enumerate() {
        let x = layout.lane_x(lane);
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.04, 0.04, 0.05),
                Vec2::new(layout.lane_w - 4.0 * layout.scale, layout.lane_height),
            ),
            Transform::from_xyz(x, layout.lane_center_y, -4.0),
            LaneColumn { lane },
            GameplayEntity,
        ));
        let receptor_w = layout.lane_w - 10.0 * layout.scale;
        let receptor_h = REF_RECEPTOR_H * layout.scale;
        commands.spawn((
            Sprite::from_color(
                lane_receptor_color(lane, 0.0),
                Vec2::new(receptor_w, receptor_h),
            ),
            Transform::from_xyz(x, layout.judge_y, 1.0),
            LaneReceptor { lane },
            LaneReceptorFlash {
                timer: Timer::from_seconds(0.0, TimerMode::Once),
            },
            GameplayEntity,
        ));
        commands.spawn((
            Text2d::new(format!(
                "{}\n{}/{}",
                spec.label, spec.gm_melodic_key, spec.gm_drum_key
            )),
            TextFont::from_font_size(18.0 * layout.scale),
            TextColor(spec.color),
            TextLayout::new_with_justify(Justify::Center),
            Transform::from_xyz(x, layout.judge_y - layout.label_offset_y, 5.0),
            LaneLabel { lane },
            ScaledFontSize(18.0),
            GameplayEntity,
        ));
    }

    keyboard_viz::spawn_key_caps(&mut commands, &layout, &bindings);
    spawn_gauge_bar(&mut commands, &layout);

    commands.spawn((
        Sprite::from_color(
            Color::srgb(1.0, 0.92, 0.2),
            Vec2::new(layout.judge_line_width, 2.0 * layout.scale),
        ),
        Transform::from_xyz(0.0, layout.judge_y, 2.0),
        JudgeLine,
        GameplayEntity,
    ));

    let (min_time, max_time) =
        layout.visible_chart_time_window(clock.visual_elapsed, run.lane_speed);
    let mut streams = PlayfieldVisualStreams::default();
    streams.notes.align_to_time(&chart.notes, min_time);
    streams
        .metronome
        .align_to_time(&chart.metronome_beats, min_time);
    streams.notes.spawn_visible_through(
        &mut commands,
        &chart.notes,
        &layout,
        &clock,
        &run,
        max_time,
    );
    commands.insert_resource(streams);

    spawn_gameplay_hud(&mut commands, &fonts);
}

fn spawn_gameplay_hud(commands: &mut Commands, fonts: &UiFonts) {
    commands.spawn((
        screen_root(),
        ZIndex(20),
        GameplayEntity,
        GameplayHudRoot,
        children![
            (
                Node {
                    width: percent(100),
                    padding: UiRect::axes(px(SPACING_MD), px(SPACING_SM)),
                    flex_direction: FlexDirection::Column,
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
                        BackgroundColor(BG_OVERLAY),
                        BorderColor::all(BORDER_SUBTLE),
                        children![
                            (
                                Text::new("Score 0000000"),
                                text_font(fonts, FONT_HUD),
                                TextColor(TEXT_PRIMARY),
                                GameplayHudScore,
                            ),
                            (
                                Text::new("Acc 0.00%"),
                                text_font(fonts, FONT_HUD),
                                TextColor(TEXT_ACCENT),
                                GameplayHudAccuracy,
                            ),
                            (
                                Text::new("Combo 0 / Max 0"),
                                text_font(fonts, FONT_HUD),
                                TextColor(TEXT_PRIMARY),
                                GameplayHudCombo,
                            ),
                            (
                                Text::new("P:0 G:0 Good:0 Poor:0 Miss:0"),
                                text_font(fonts, FONT_CAPTION),
                                TextColor(TEXT_MUTED),
                                GameplayHudCounters,
                            ),
                        ],
                    ),
                    (
                        Node {
                            width: px(220.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(SPACING_SM),
                            padding: UiRect::all(px(SPACING_SM)),
                            border: UiRect::all(px(1.0)),
                            border_radius: BorderRadius::all(px(8.0)),
                            ..default()
                        },
                        BackgroundColor(BG_OVERLAY),
                        BorderColor::all(BORDER_SUBTLE),
                        children![
                            (
                                Text::new("Gauge 80%"),
                                text_font(fonts, FONT_CAPTION),
                                TextColor(TEXT_SECONDARY),
                                GameplayHudGauge,
                            ),
                            (
                                Node {
                                    width: px(200.0),
                                    height: px(10.0),
                                    border_radius: BorderRadius::all(px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(BG_ELEVATED),
                                children![(
                                    Node {
                                        width: percent(80.0),
                                        height: percent(100),
                                        border_radius: BorderRadius::all(px(5.0)),
                                        ..default()
                                    },
                                    BackgroundColor(SUCCESS),
                                    GameplayHudGaugeFill,
                                )],
                            ),
                        ],
                    ),
                ],
            ),
            (
                Node {
                    width: percent(100),
                    height: percent(100),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
                children![(
                    Node {
                        min_width: px(180.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(px(SPACING_MD)),
                        border: UiRect::all(px(1.0)),
                        border_radius: BorderRadius::all(px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.72)),
                    BorderColor::all(BORDER_SUBTLE),
                    children![(
                        Text::new("READY"),
                        text_font(fonts, FONT_HEADING),
                        TextColor(TEXT_ACCENT),
                        JudgementText,
                        GameplayHudJudgement,
                    ),],
                ),],
            ),
            (
                Node {
                    width: px(640.0),
                    position_type: PositionType::Absolute,
                    left: px(SPACING_MD),
                    bottom: px(SPACING_MD),
                    padding: UiRect::all(px(SPACING_SM)),
                    border: UiRect::all(px(1.0)),
                    border_radius: BorderRadius::all(px(8.0)),
                    ..default()
                },
                BackgroundColor(BG_OVERLAY),
                BorderColor::all(BORDER_SUBTLE),
                Visibility::Hidden,
                GameplayHudDebug,
                children![(
                    Text::new(""),
                    text_font(fonts, FONT_CAPTION),
                    TextColor(TEXT_SECONDARY),
                    GameplayHudDebugText,
                ),],
            ),
        ],
    ));
}

pub(crate) fn cleanup_gameplay(
    mut commands: Commands,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    bgm_instance: Option<Res<BgmInstance>>,
    mut active: ResMut<ActiveSounds>,
    mut metronome_active: ResMut<MetronomeActive>,
    entities: Query<Entity, (With<GameplayEntity>, Without<ChildOf>)>,
) {
    stop_all_playback(
        &mut commands,
        &mut audio_instances,
        bgm_instance,
        &mut active,
        &mut metronome_active,
    );

    for entity in &entities {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<Chart>();
    commands.remove_resource::<ChartTiming>();
    commands.remove_resource::<SoundBank>();
    commands.remove_resource::<MetronomeSounds>();
    commands.remove_resource::<BackgroundDecodeReceiver>();
}
