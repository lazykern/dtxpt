use bevy::prelude::*;

pub const BG_PRIMARY: Color = Color::srgb(0.06, 0.08, 0.12);
pub const BG_SECONDARY: Color = Color::srgb(0.10, 0.12, 0.18);
pub const BG_ELEVATED: Color = Color::srgb(0.14, 0.17, 0.24);
pub const BG_OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.72);

pub const TEXT_PRIMARY: Color = Color::srgb(0.92, 0.94, 0.98);
pub const TEXT_SECONDARY: Color = Color::srgb(0.62, 0.68, 0.78);
pub const TEXT_MUTED: Color = Color::srgb(0.45, 0.50, 0.58);
pub const TEXT_ACCENT: Color = Color::srgb(0.55, 0.85, 1.0);

pub const ACCENT: Color = Color::srgb(0.28, 0.55, 0.85);
pub const ACCENT_HOVER: Color = Color::srgb(0.35, 0.65, 0.95);
pub const ACCENT_PRESSED: Color = Color::srgb(0.20, 0.45, 0.72);

pub const SUCCESS: Color = Color::srgb(0.35, 0.85, 0.55);
pub const WARNING: Color = Color::srgb(0.95, 0.75, 0.30);
pub const DANGER: Color = Color::srgb(0.95, 0.35, 0.35);

pub const BORDER_SUBTLE: Color = Color::srgba(0.28, 0.42, 0.58, 0.45);
pub const BORDER_FOCUS: Color = Color::srgba(0.45, 0.70, 0.95, 0.85);

pub const BUTTON_NORMAL: Color = Color::srgb(0.16, 0.20, 0.28);
pub const BUTTON_HOVER: Color = Color::srgb(0.22, 0.28, 0.38);
pub const BUTTON_PRESSED: Color = Color::srgb(0.12, 0.32, 0.48);

pub const CARD_NORMAL: Color = Color::srgba(0.12, 0.15, 0.22, 0.65);
pub const CARD_SELECTED: Color = Color::srgba(0.18, 0.30, 0.48, 0.85);

pub const RANK_SS: Color = Color::srgb(1.0, 0.85, 0.25);
pub const RANK_S: Color = Color::srgb(0.85, 0.90, 0.95);
pub const RANK_A: Color = Color::srgb(0.45, 0.90, 0.55);
pub const RANK_B: Color = Color::srgb(0.55, 0.75, 1.0);
pub const RANK_C: Color = Color::srgb(0.75, 0.75, 0.80);

pub fn rank_color(rank: &str) -> Color {
    match rank {
        "SS" => RANK_SS,
        "S" => RANK_S,
        "A" => RANK_A,
        "B" => RANK_B,
        _ => RANK_C,
    }
}
