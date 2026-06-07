use bevy::prelude::*;

use super::palette::*;
use super::widgets::UiButton;

#[derive(Component)]
pub struct ScreenFadeIn {
    pub timer: Timer,
    pub start_alpha: f32,
}

#[derive(Component)]
pub struct SlideIn {
    pub timer: Timer,
    pub offset: f32,
}

pub fn screen_fade_in(duration: f32) -> ScreenFadeIn {
    ScreenFadeIn {
        timer: Timer::from_seconds(duration, TimerMode::Once),
        start_alpha: 0.0,
    }
}

pub fn slide_in(duration: f32, offset: f32) -> SlideIn {
    SlideIn {
        timer: Timer::from_seconds(duration, TimerMode::Once),
        offset,
    }
}

pub fn update_screen_fade_in(
    time: Res<Time>,
    mut query: Query<(&mut ScreenFadeIn, &mut BackgroundColor)>,
) {
    for (mut fade, mut bg) in &mut query {
        fade.timer.tick(time.delta());
        let t = fade.timer.fraction().clamp(0.0, 1.0);
        let alpha = fade.start_alpha + (1.0 - fade.start_alpha) * t;
        let base = bg.0.to_srgba();
        bg.0 = Color::srgba(base.red, base.green, base.blue, alpha);
        if fade.timer.is_finished() {
            bg.0 = Color::srgba(base.red, base.green, base.blue, 1.0);
        }
    }
}

pub fn update_button_interactions(
    mut query: Query<
        (
            &Interaction,
            &UiButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, style, mut bg, mut border) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                bg.0 = style.pressed;
                *border = BorderColor::all(ACCENT);
            }
            Interaction::Hovered => {
                bg.0 = style.hovered;
                *border = BorderColor::all(BORDER_FOCUS);
            }
            Interaction::None => {
                bg.0 = style.normal;
                *border = BorderColor::all(BORDER_SUBTLE);
            }
        }
    }
}
