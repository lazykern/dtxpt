use bevy::prelude::*;

use crate::app::markers::StageScreen;
use crate::app::state::AppState;
use crate::ui::fonts::{UiFonts, text_font};

/// Time the StageClear / StageFailed banner stays on screen before
/// auto-advancing to the Result screen. Mirrors the visual duration
/// BocuD's `CActPerformanceStageClear` /
/// `CActPerformanceStageFailed` hold the interstitial — typically
/// 1.5–2.5 seconds of artwork + SFX. dtxpt's first pass ships the
/// banner + auto-advance without the full artwork animation strip.
const STAGE_SCREEN_SECONDS: f32 = 1.8;

#[derive(Resource)]
pub struct StageScreenTimer(pub Timer);

pub fn setup_stage_clear(
    mut commands: Commands,
    fonts: Res<UiFonts>,
) {
    commands.spawn((
        stage_screen_root(),
        StageScreen,
        children![(
            Node {
                width: percent(80),
                height: px(140.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![(
                Text::new("STAGE CLEAR!"),
                text_font(&fonts, 56.0),
                TextColor(Color::srgb(0.55, 0.95, 0.65)),
            )],
        )],
    ));
    commands.insert_resource(StageScreenTimer(Timer::from_seconds(
        STAGE_SCREEN_SECONDS,
        TimerMode::Once,
    )));
}

pub fn setup_stage_failed(
    mut commands: Commands,
    fonts: Res<UiFonts>,
) {
    commands.spawn((
        stage_screen_root(),
        StageScreen,
        children![(
            Node {
                width: percent(80),
                height: px(140.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![(
                Text::new("STAGE FAILED"),
                text_font(&fonts, 56.0),
                TextColor(Color::srgb(0.95, 0.40, 0.40)),
            )],
        )],
    ));
    commands.insert_resource(StageScreenTimer(Timer::from_seconds(
        STAGE_SCREEN_SECONDS,
        TimerMode::Once,
    )));
}

pub fn stage_clear_auto_advance(
    time: Res<Time>,
    timer: Option<ResMut<StageScreenTimer>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    stage_query: Query<Entity, With<StageScreen>>,
) {
    let Some(mut timer) = timer else {
        return;
    };
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        for entity in &stage_query {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<StageScreenTimer>();
        next_state.set(AppState::Result);
    }
}

fn stage_screen_root() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        right: Val::Px(0.0),
        top: Val::Px(0.0),
        bottom: Val::Px(0.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        row_gap: px(24.0),
        ..default()
    }
}