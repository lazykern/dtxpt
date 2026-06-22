use bevy::prelude::*;

use crate::app::markers::{ResultAction, ResultActionButton, ResultScreen};
use crate::app::state::AppState;
use crate::gameplay::RunResult;
use crate::persistence::ScoreStore;
use crate::ui::fonts::{UiFonts, text_font};
use crate::ui::palette::*;
use crate::ui::theme::*;
use crate::ui::widgets::*;

pub fn setup_result(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    result: Option<Res<RunResult>>,
    scores: Res<ScoreStore>,
) {
    let (
        rank,
        clear_line,
        title,
        source,
        score,
        accuracy,
        max_combo,
        gauge,
        best_line,
        judgements,
        combo_flags,
        instrument_ranks,
    ) = if let Some(result) = result {
        let best = scores.scores.get(&result.chart_path);
        let best_line = best
            .map(|best| format!("{:07}  {:.2}%", best.score, best.accuracy))
            .unwrap_or_else(|| "none".to_string());
        let instrument_ranks = best
            .map(|best| best.instrument_ranks())
            .unwrap_or_default();
        let clear_line = if result.failed {
            "FAILED".to_string()
        } else if result.practice {
            "Practice".to_string()
        } else if !result.auto_lanes.is_empty() {
            "Auto Assist".to_string()
        } else if result.cleared {
            "CLEAR".to_string()
        } else {
            "CLEAR?".to_string()
        };
        let judged = result.perfect + result.great + result.good + result.poor + result.miss;
        let all_perfect = judged > 0 && result.perfect == judged;
        let combo_flags = [
            result.full_combo.then_some("FULL COMBO"),
            all_perfect.then_some("ALL PERFECT"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("   ");
        (
            result.rank.clone(),
            clear_line,
            result.title.clone(),
            result.source.clone(),
            format!("{:07}", result.score),
            format!("{:.2}%", result.accuracy),
            result.max_combo.to_string(),
            format!("{:.0}%", result.gauge * 100.0),
            best_line,
            (
                result.perfect,
                result.great,
                result.good,
                result.poor,
                result.miss,
            ),
            combo_flags,
            instrument_ranks,
        )
    } else {
        (
            "—".into(),
            "No result".into(),
            "—".into(),
            "—".into(),
            "0000000".into(),
            "0.00%".into(),
            "0".into(),
            "0%".into(),
            "none".into(),
            (0, 0, 0, 0, 0),
            String::new(),
            <[(&'static str, &str); 3]>::default(),
        )
    };

    let rank_color = rank_color(&rank);

    commands.spawn((
        screen_root(),
        ResultScreen,
        children![(
            centered_column(SPACING_LG),
            children![
                (
                    Node {
                        width: px(640.0),
                        flex_direction: FlexDirection::Column,
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
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(SPACING_LG),
                                ..default()
                            },
                            children![
                                (
                                    Text::new(&rank),
                                    text_font(&fonts, 64.0),
                                    TextColor(rank_color),
                                ),
                                (
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        row_gap: px(SPACING_XS),
                                        flex_grow: 1.0,
                                        ..default()
                                    },
                                    children![
                                        (
                                            Text::new(&title),
                                            text_font(&fonts, FONT_HEADING),
                                            TextColor(TEXT_ACCENT),
                                        ),
                                        (
                                            Text::new(&source),
                                            text_font(&fonts, FONT_BODY),
                                            TextColor(TEXT_SECONDARY),
                                        ),
                                        (
                                            Text::new(&clear_line),
                                            text_font(&fonts, FONT_BODY),
                                            TextColor(if clear_line == "FAILED" {
                                                DANGER
                                            } else {
                                                SUCCESS
                                            }),
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        stat_row_bundle(&fonts, "Score", &score),
                        stat_row_bundle(&fonts, "Accuracy", &accuracy),
                        stat_row_bundle(&fonts, "Max Combo", &max_combo),
                        stat_row_bundle(&fonts, "Gauge", &gauge),
                        stat_row_bundle(&fonts, "Best", &best_line),
                        per_instrument_ranks_bundle(&fonts, instrument_ranks),
                        (
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: px(SPACING_LG),
                                row_gap: px(SPACING_SM),
                                ..default()
                            },
                            children![(
                                Text::new(format!(
                                    "P:{}  G:{}  Good:{}  Poor:{}  Miss:{}",
                                    judgements.0,
                                    judgements.1,
                                    judgements.2,
                                    judgements.3,
                                    judgements.4
                                )),
                                text_font(&fonts, FONT_BODY),
                                TextColor(TEXT_PRIMARY),
                            ),],
                        ),
                        (
                            Text::new(if combo_flags.is_empty() {
                                " "
                            } else {
                                &combo_flags
                            }),
                            text_font(&fonts, FONT_BODY),
                            TextColor(if combo_flags.is_empty() {
                                TEXT_MUTED
                            } else {
                                WARNING
                            }),
                        ),
                        (
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                column_gap: px(SPACING_MD),
                                margin: UiRect::top(px(SPACING_MD)),
                                ..default()
                            },
                            children![
                                (
                                    button_bundle(&fonts, "Retry (R)", 200.0, 48.0),
                                    ResultActionButton(ResultAction::Retry),
                                ),
                                (
                                    button_bundle(&fonts, "Song Select (Enter)", 260.0, 48.0),
                                    ResultActionButton(ResultAction::SongSelect),
                                ),
                            ],
                        ),
                    ],
                ),
                (
                    Text::new("F1  Settings"),
                    text_font(&fonts, FONT_CAPTION),
                    TextColor(TEXT_MUTED),
                ),
            ],
        )],
    ));
}

pub(crate) fn result_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    buttons: Query<(&Interaction, &ResultActionButton), Changed<Interaction>>,
) {
    for (interaction, action) in &buttons {
        if *interaction == Interaction::Pressed {
            match action.0 {
                ResultAction::Retry => next_state.set(AppState::LoadingSong),
                ResultAction::SongSelect => next_state.set(AppState::SongSelect),
            }
        }
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(AppState::LoadingSong);
    }
    if keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Escape)
    {
        next_state.set(AppState::SongSelect);
    }
}

/// Render the per-instrument best-rank row. Each instrument shows its
/// rank label, or "—" when the player has never played the chart on
/// that instrument.
fn per_instrument_ranks_bundle(
    fonts: &UiFonts,
    ranks: [(&'static str, &str); 3],
) -> impl Bundle {
    let any = ranks.iter().any(|(_, r)| !r.is_empty());
    let d0 = if ranks[0].1.is_empty() { "—" } else { ranks[0].1 };
    let d1 = if ranks[1].1.is_empty() { "—" } else { ranks[1].1 };
    let d2 = if ranks[2].1.is_empty() { "—" } else { ranks[2].1 };
    (
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(SPACING_LG),
            row_gap: px(SPACING_SM),
            align_items: AlignItems::Center,
            display: if any { Display::Flex } else { Display::None },
            ..default()
        },
        children![
            caption_bundle(fonts, "Best ranks:", TEXT_SECONDARY),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(SPACING_XS),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    caption_bundle(fonts, ranks[0].0, TEXT_SECONDARY),
                    caption_bundle(fonts, d0, TEXT_ACCENT),
                ],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(SPACING_XS),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    caption_bundle(fonts, ranks[1].0, TEXT_SECONDARY),
                    caption_bundle(fonts, d1, TEXT_ACCENT),
                ],
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(SPACING_XS),
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    caption_bundle(fonts, ranks[2].0, TEXT_SECONDARY),
                    caption_bundle(fonts, d2, TEXT_ACCENT),
                ],
            ),
        ],
    )
}
