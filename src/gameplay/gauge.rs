#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use dtxpt::chart::Judgement;

use crate::app::markers::{GaugeBarFill, GaugeBarTrack};
use crate::config::DamageLevel;
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

/// Per-judgement base delta, before damage level. Same as `gauge_delta`
/// but exposed as the "raw" value for clarity in DamageLevel math.
fn raw_delta(judgement: Judgement) -> f32 {
    gauge_delta(judgement)
}

/// Damage level multiplier for Poor / Miss. BocuD `EDamageLevel`.
/// Small = half damage; High = double damage.
pub fn damage_multiplier(level: DamageLevel) -> f32 {
    match level {
        DamageLevel::Small => 0.5,
        DamageLevel::Normal => 1.0,
        DamageLevel::High => 2.0,
    }
}

/// Apply a judgement to the gauge. Honours:
/// - Damage level (multiplies Poor/Miss negative delta)
/// - Risky mode (decrements `risky_times_remaining` on Miss; fails when 0)
/// - Auto-add-gage (adds a small positive delta on autoplayed chips)
/// - Practice mode (no fail, no clamp-to-fail)
pub fn apply_gauge(run: &mut RunState, judgement: Judgement, autoplayed: bool) {
    let raw = raw_delta(judgement);
    let delta = if matches!(judgement, Judgement::Poor | Judgement::Miss) {
        raw * damage_multiplier(run.damage_level)
    } else {
        raw
    };
    let auto_bonus = if autoplayed
        && run.auto_add_gauge
        && matches!(
            judgement,
            Judgement::Perfect | Judgement::Great | Judgement::Good
        ) {
        0.001
    } else {
        0.0
    };
    run.gauge = (run.gauge + delta + auto_bonus).clamp(0.0, 1.0);
    if run.practice {
        return;
    }
    // Risky: every Miss decrements the counter; reaching 0 fails the run
    // regardless of gauge value.
    if run.risky_initial > 0 && judgement == Judgement::Miss {
        run.risky_times_remaining = run.risky_times_remaining.saturating_sub(1);
        if run.risky_times_remaining == 0 {
            run.failed = true;
            run.finished = true;
            run.last_message = "FAILED (RISKY)".into();
            return;
        }
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
            apply_gauge(&mut run, Judgement::Miss, false);
        }
        assert_eq!(run.gauge, 0.0);
        assert!(!run.failed);
        assert!(!run.finished);
    }

    #[test]
    fn normal_mode_fails_at_zero() {
        let mut run = RunState::from_config(&GameConfig::default());
        run.gauge = 0.05;
        apply_gauge(&mut run, Judgement::Miss, false);
        assert!(run.failed);
        assert!(run.finished);
    }

    #[test]
    fn risky_mode_fails_after_n_misses() {
        let cfg = GameConfig {
            gauge: crate::config::GaugeConfig {
                risky_initial: 3,
                ..crate::config::GaugeConfig::default()
            },
            ..GameConfig::default()
        };
        let mut run = RunState::from_config(&cfg);
        assert_eq!(run.risky_initial, 3);
        assert_eq!(run.risky_times_remaining, 3);
        // Three Misses → fail.
        apply_gauge(&mut run, Judgement::Miss, false);
        assert!(!run.failed);
        assert_eq!(run.risky_times_remaining, 2);
        apply_gauge(&mut run, Judgement::Miss, false);
        assert!(!run.failed);
        assert_eq!(run.risky_times_remaining, 1);
        apply_gauge(&mut run, Judgement::Miss, false);
        assert!(run.failed);
        assert!(run.finished);
        assert_eq!(run.last_message, "FAILED (RISKY)");
    }

    #[test]
    fn risky_off_uses_gauge_for_fail() {
        let mut run = RunState::from_config(&GameConfig::default());
        assert_eq!(run.risky_initial, 0);
        run.gauge = 0.5;
        apply_gauge(&mut run, Judgement::Miss, false);
        assert!(!run.failed);
    }

    #[test]
    fn damage_level_small_halves_poor_miss_penalty() {
        let cfg = GameConfig {
            gauge: crate::config::GaugeConfig {
                damage_level: DamageLevel::Small,
                ..crate::config::GaugeConfig::default()
            },
            ..GameConfig::default()
        };
        let mut run_small = RunState::from_config(&cfg);
        let mut run_normal = RunState::from_config(&GameConfig::default());
        run_small.gauge = 1.0;
        run_normal.gauge = 1.0;
        apply_gauge(&mut run_small, Judgement::Miss, false);
        apply_gauge(&mut run_normal, Judgement::Miss, false);
        // Small damage: -0.03; Normal: -0.06.
        assert!((run_small.gauge - (1.0 - 0.03)).abs() < 0.001);
        assert!((run_normal.gauge - (1.0 - 0.06)).abs() < 0.001);
    }

    #[test]
    fn damage_level_high_doubles_poor_miss_penalty() {
        let cfg = GameConfig {
            gauge: crate::config::GaugeConfig {
                damage_level: DamageLevel::High,
                ..crate::config::GaugeConfig::default()
            },
            ..GameConfig::default()
        };
        let mut run = RunState::from_config(&cfg);
        run.gauge = 1.0;
        apply_gauge(&mut run, Judgement::Miss, false);
        // Miss base -0.06, High = -0.12.
        assert!((run.gauge - 0.88).abs() < 0.001);
    }

    #[test]
    fn auto_add_gauge_only_applies_to_autoplayed_chips() {
        let cfg = GameConfig {
            gauge: crate::config::GaugeConfig {
                auto_add_gauge: true,
                ..crate::config::GaugeConfig::default()
            },
            ..GameConfig::default()
        };
        let mut run = RunState::from_config(&cfg);
        run.gauge = 0.5;
        // Perfect hit by player: +0.005, no auto bonus.
        apply_gauge(&mut run, Judgement::Perfect, false);
        assert!((run.gauge - 0.505).abs() < 0.001);
        // Perfect by autoplay: +0.005 + 0.001 auto bonus = +0.006 over 0.505.
        apply_gauge(&mut run, Judgement::Perfect, true);
        assert!((run.gauge - 0.511).abs() < 0.001);
    }
}
