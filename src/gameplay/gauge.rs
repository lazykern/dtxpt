#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use dtxpt::chart::Judgement;

use crate::app::markers::{GaugeBarFill, GaugeBarTrack};
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;

pub const GAUGE_START: f32 = 0.80;
pub const GAUGE_CLEAR: f32 = 0.80;

pub fn gauge_delta(judgement: Judgement) -> f32 {
    match judgement {
        Judgement::Perfect => 0.005,
        Judgement::Great => 0.002,
        Judgement::Good => 0.0,
        Judgement::Poor => -0.03,
        Judgement::Miss => -0.06,
    }
}

pub fn apply_gauge(run: &mut RunState, judgement: Judgement) {
    run.gauge = (run.gauge + gauge_delta(judgement)).clamp(0.0, 1.0);
    if run.practice {
        return;
    }
    if run.gauge <= 0.0 {
        run.gauge = 0.0;
        run.failed = true;
        run.finished = true;
        run.last_message = "FAILED".into();
    }
}

pub fn gauge_fill_color(gauge: f32, failed: bool) -> Color {
    if failed {
        return Color::srgb(0.95, 0.2, 0.25);
    }
    if gauge >= GAUGE_CLEAR {
        Color::srgb(0.25, 0.9, 0.45)
    } else if gauge >= 0.4 {
        Color::srgb(0.95, 0.85, 0.2)
    } else {
        Color::srgb(0.95, 0.45, 0.2)
    }
}

pub fn spawn_gauge_bar(commands: &mut Commands, layout: &PlayfieldLayout) {
    let width = layout.gauge_bar_width();
    let height = layout.gauge_bar_height();
    let y = layout.gauge_bar_y();

    commands.spawn((
        Sprite::from_color(
            Color::srgba(0.08, 0.08, 0.1, 0.85),
            Vec2::new(width, height),
        ),
        Transform::from_xyz(0.0, y, 8.0),
        GaugeBarTrack,
        crate::app::markers::GameplayEntity,
    ));
    commands.spawn((
        Sprite::from_color(
            gauge_fill_color(GAUGE_START, false),
            Vec2::new(width, height),
        ),
        Transform::from_xyz(-width * 0.5, y, 9.0),
        GaugeBarFill,
        crate::app::markers::GameplayEntity,
    ));
}

pub(crate) fn update_gauge_bar(
    run: Res<RunState>,
    layout: Res<PlayfieldLayout>,
    mut bars: ParamSet<(
        Query<(&mut Sprite, &mut Transform), With<GaugeBarTrack>>,
        Query<(&mut Sprite, &mut Transform), With<GaugeBarFill>>,
    )>,
) {
    if !layout.is_changed() && !run.is_changed() {
        return;
    }

    let width = layout.gauge_bar_width();
    let height = layout.gauge_bar_height();
    let y = layout.gauge_bar_y();
    let fill_w = width * run.gauge.max(0.0);

    if let Ok((mut sprite, mut transform)) = bars.p0().single_mut() {
        sprite.custom_size = Some(Vec2::new(width, height));
        transform.translation.y = y;
    }

    if let Ok((mut sprite, mut transform)) = bars.p1().single_mut() {
        sprite.color = gauge_fill_color(run.gauge, run.failed);
        sprite.custom_size = Some(Vec2::new(fill_w.max(1.0), height));
        transform.translation.x = -width * 0.5 + fill_w * 0.5;
        transform.translation.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;
    use crate::gameplay::run::RunState;

    #[test]
    fn practice_mode_tracks_gauge_without_failing() {
        let mut run = RunState::from_config(&GameConfig::default());
        run.practice = true;
        for _ in 0..20 {
            apply_gauge(&mut run, Judgement::Miss);
        }
        assert_eq!(run.gauge, 0.0);
        assert!(!run.failed);
        assert!(!run.finished);
    }

    #[test]
    fn normal_mode_fails_at_zero() {
        let mut run = RunState::from_config(&GameConfig::default());
        run.gauge = 0.05;
        apply_gauge(&mut run, Judgement::Miss);
        assert!(run.failed);
        assert!(run.finished);
    }
}
