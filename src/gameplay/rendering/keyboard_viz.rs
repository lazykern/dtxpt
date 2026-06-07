use bevy::prelude::*;
use dtxpt::input::lanes::{LANE_DISPLAY_ORDER, LANES};
use dtxpt::input::{InputBindings, keycode_display_name};

use crate::app::markers::{GameplayEntity, ScaledFontSize};
use crate::gameplay::PlayfieldLayout;

const KEY_CAP_FLASH_SECS: f32 = 0.08;
const KEY_CAP_BASE: Color = Color::srgb(0.07, 0.07, 0.09);

#[derive(Component)]
pub struct KeyCap {
    pub lane: usize,
}

#[derive(Component)]
pub struct KeyCapLabel;

#[derive(Component)]
pub struct KeyCapFlash {
    pub timer: Timer,
}

impl Default for KeyCapFlash {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }
}

pub fn spawn_key_caps(commands: &mut Commands, layout: &PlayfieldLayout, bindings: &InputBindings) {
    let cap_w = layout.key_cap_w();
    let cap_h = layout.key_cap_h();

    for lane in LANE_DISPLAY_ORDER {
        let spec = &LANES[lane];
        let x = layout.lane_x(lane);
        let y = layout.key_viz_y();

        commands.spawn((
            Sprite::from_color(key_cap_color(lane, 0.0), Vec2::new(cap_w, cap_h)),
            Transform::from_xyz(x, y, 6.0),
            KeyCap { lane },
            KeyCapFlash::default(),
            GameplayEntity,
        ));

        commands.spawn((
            Text2d::new(format!(
                "{}\n{}/{}\n[{}]",
                spec.label,
                spec.gm_melodic_key,
                spec.gm_drum_key,
                keyboard_summary(bindings, lane)
            )),
            TextFont::from_font_size(13.0 * layout.scale),
            TextColor(spec.color),
            TextLayout::new_with_justify(Justify::Center),
            Transform::from_xyz(x, y, 7.0),
            KeyCapLabel,
            KeyCap { lane },
            ScaledFontSize(13.0),
            GameplayEntity,
        ));
    }
}

pub fn apply_key_cap_sprites(
    layout: &PlayfieldLayout,
    caps: &mut Query<(&KeyCap, &mut Sprite, &mut Transform), Without<KeyCapLabel>>,
) {
    let cap_w = layout.key_cap_w();
    let cap_h = layout.key_cap_h();
    let y = layout.key_viz_y();

    for (cap, mut sprite, mut transform) in caps.iter_mut() {
        sprite.custom_size = Some(Vec2::new(cap_w, cap_h));
        transform.translation.x = layout.lane_x(cap.lane);
        transform.translation.y = y;
    }
}

pub(crate) fn apply_key_cap_labels(
    layout: &PlayfieldLayout,
    labels: &mut Query<
        (&KeyCap, &mut Transform, &ScaledFontSize, &mut TextFont),
        With<KeyCapLabel>,
    >,
) {
    let y = layout.key_viz_y();

    for (cap, mut transform, base_font, mut font) in labels.iter_mut() {
        transform.translation.x = layout.lane_x(cap.lane);
        transform.translation.y = y;
        font.font_size = base_font.0 * layout.scale;
    }
}

pub fn flash_key_cap(
    lane: usize,
    caps: &mut Query<(&KeyCap, &mut Sprite, &mut KeyCapFlash), Without<KeyCapLabel>>,
) {
    for (cap, mut sprite, mut flash) in caps.iter_mut() {
        if cap.lane != lane {
            continue;
        }
        flash.timer = Timer::from_seconds(KEY_CAP_FLASH_SECS, TimerMode::Once);
        sprite.color = key_cap_color(lane, 1.0);
        break;
    }
}

pub fn update_key_cap_flashes(
    time: Res<Time>,
    mut caps: Query<(&KeyCap, &mut Sprite, &mut KeyCapFlash), Without<KeyCapLabel>>,
) {
    for (cap, mut sprite, mut flash) in caps.iter_mut() {
        if flash.timer.duration().as_secs_f32() == 0.0 {
            continue;
        }
        flash.timer.tick(time.delta());
        let strength = if flash.timer.is_finished() {
            0.0
        } else {
            1.0 - flash.timer.fraction()
        };
        sprite.color = key_cap_color(cap.lane, strength);
    }
}

fn keyboard_summary(bindings: &InputBindings, lane: usize) -> String {
    let keys = bindings.keyboard_keys_for_lane(lane);
    match keys.as_slice() {
        [] => "—".to_string(),
        [only] => keycode_display_name(*only).to_string(),
        [first, second] => format!(
            "{}/{}",
            keycode_display_name(*first),
            keycode_display_name(*second)
        ),
        [first, rest @ ..] => format!("{} +{}", keycode_display_name(*first), rest.len()),
    }
}

fn key_cap_color(lane: usize, strength: f32) -> Color {
    let s = strength.clamp(0.0, 1.0);
    let tint = LANES[lane].color.to_srgba();
    let base = KEY_CAP_BASE.to_srgba();
    let mix = 0.12 + 0.55 * s;
    let lift = 0.08 * s;
    Color::srgb(
        (base.red + tint.red * mix + lift).min(1.0),
        (base.green + tint.green * mix + lift).min(1.0),
        (base.blue + tint.blue * mix + lift).min(1.0),
    )
}
