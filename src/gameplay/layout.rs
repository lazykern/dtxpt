#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};

use dtxpt::input::lanes::{LANES, lane_display_slot};

use crate::app::markers::*;
use crate::gameplay::constants::HUD_PADDING;
use crate::gameplay::rendering::keyboard_viz;
use crate::ui::theme::{REF_RECEPTOR_H, *};

#[derive(Resource, Clone, Copy)]
pub struct PlayfieldLayout {
    pub scale: f32,
    pub window_half_w: f32,
    pub window_half_h: f32,
    pub judge_y: f32,
    pub lane_w: f32,
    pub note_w: f32,
    pub note_h: f32,
    pub backboard_size: Vec2,
    pub backboard_center_y: f32,
    pub lane_height: f32,
    pub lane_center_y: f32,
    pub label_offset_y: f32,
    pub judge_line_width: f32,
    pub metronome_line_height: f32,
    pub note_fade_span: f32,
    pub judgement_center_y: f32,
    pub hud_padding: f32,
}

impl Default for PlayfieldLayout {
    fn default() -> Self {
        Self::from_size(REF_WIDTH, REF_HEIGHT)
    }
}

impl PlayfieldLayout {
    pub fn from_window(window: &Window) -> Self {
        Self::from_size(window.width(), window.height())
    }

    pub fn from_size(width: f32, height: f32) -> Self {
        let scale = (width / REF_WIDTH).min(height / REF_HEIGHT);
        Self {
            scale,
            window_half_w: width * 0.5,
            window_half_h: height * 0.5,
            judge_y: REF_JUDGE_Y * scale,
            lane_w: REF_LANE_W * scale,
            note_w: REF_NOTE_W * scale,
            note_h: REF_NOTE_H * scale,
            backboard_size: Vec2::new(REF_BACKBOARD_W * scale, REF_BACKBOARD_H * scale),
            backboard_center_y: REF_BACKBOARD_Y * scale,
            lane_height: REF_LANE_H * scale,
            lane_center_y: REF_LANE_Y * scale,
            label_offset_y: REF_LABEL_OFFSET_Y * scale,
            judge_line_width: REF_JUDGE_LINE_W * scale,
            metronome_line_height: REF_METRONOME_LINE_H * scale,
            note_fade_span: REF_NOTE_FADE_SPAN * scale,
            judgement_center_y: REF_JUDGEMENT_Y * scale,
            hud_padding: HUD_PADDING * scale,
        }
    }

    pub fn lane_x(&self, lane: usize) -> f32 {
        let slot = lane_display_slot(lane) as f32;
        let total = LANES.len() as f32;
        (slot - (total - 1.0) * 0.5) * self.lane_w
    }

    pub fn key_viz_y(&self) -> f32 {
        REF_KEY_VIZ_Y * self.scale
    }

    pub fn key_cap_w(&self) -> f32 {
        REF_KEY_CAP_W * self.scale
    }

    pub fn key_cap_h(&self) -> f32 {
        REF_KEY_CAP_H * self.scale
    }

    pub fn note_y(&self, note_time: f32, elapsed: f32, lane_speed: f32) -> f32 {
        self.judge_y + (note_time - elapsed) * self.scroll_px_per_sec(lane_speed)
    }

    pub fn scroll_px_per_sec(&self, lane_speed: f32) -> f32 {
        REF_BASE_SCROLL_PX_PER_SEC * self.scale * lane_speed
    }

    pub fn hud_top_left(&self) -> Vec2 {
        Vec2::new(
            -self.window_half_w + self.hud_padding,
            self.window_half_h - self.hud_padding,
        )
    }

    pub fn hud_bounds(&self, width: f32, height: f32) -> Vec2 {
        Vec2::new(
            (width - self.hud_padding * 2.0).max(200.0),
            (height - self.hud_padding * 2.0).max(200.0),
        )
    }

    pub fn gauge_bar_width(&self) -> f32 {
        REF_GAUGE_BAR_W * self.scale
    }

    pub fn gauge_bar_height(&self) -> f32 {
        REF_GAUGE_BAR_H * self.scale
    }

    pub fn gauge_bar_y(&self) -> f32 {
        REF_GAUGE_BAR_Y * self.scale
    }

    pub fn note_bar_w(&self) -> f32 {
        self.lane_w - 10.0 * self.scale
    }
}

pub fn sync_playfield_layout(
    mut resize_events: MessageReader<WindowResized>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut layout: ResMut<PlayfieldLayout>,
    mut dirty: Local<bool>,
) {
    if resize_events.read().next().is_some() {
        *dirty = true;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let next = PlayfieldLayout::from_window(window);
    if *dirty
        || next.scale != layout.scale
        || next.window_half_w != layout.window_half_w
        || next.window_half_h != layout.window_half_h
    {
        *layout = next;
        *dirty = false;
    }
}

pub(crate) fn apply_playfield_layout(
    layout: Res<PlayfieldLayout>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut playfield: ParamSet<(
        Query<(&mut Sprite, &mut Transform), With<PlayfieldBackboard>>,
        Query<(&LaneColumn, &mut Sprite, &mut Transform)>,
        Query<(&LaneReceptor, &mut Sprite, &mut Transform)>,
        Query<(&mut Sprite, &mut Transform), With<JudgeLine>>,
        Query<(&LaneLabel, &mut Transform, &ScaledFontSize, &mut TextFont)>,
    )>,
) {
    if !layout.is_changed() {
        return;
    }

    let _ = windows;

    for (mut sprite, mut transform) in playfield.p0().iter_mut() {
        sprite.custom_size = Some(layout.backboard_size);
        transform.translation.y = layout.backboard_center_y;
    }

    for (column, mut sprite, mut transform) in playfield.p1().iter_mut() {
        sprite.custom_size = Some(Vec2::new(
            layout.lane_w - 4.0 * layout.scale,
            layout.lane_height,
        ));
        transform.translation.x = layout.lane_x(column.lane);
        transform.translation.y = layout.lane_center_y;
    }

    let receptor_w = layout.lane_w - 10.0 * layout.scale;
    let receptor_h = REF_RECEPTOR_H * layout.scale;
    for (receptor, mut sprite, mut transform) in playfield.p2().iter_mut() {
        sprite.custom_size = Some(Vec2::new(receptor_w, receptor_h));
        transform.translation.x = layout.lane_x(receptor.lane);
        transform.translation.y = layout.judge_y;
    }

    for (mut sprite, mut transform) in playfield.p3().iter_mut() {
        sprite.custom_size = Some(Vec2::new(layout.judge_line_width, 2.0 * layout.scale));
        transform.translation.y = layout.judge_y;
    }

    for (label, mut transform, base_font, mut font) in playfield.p4().iter_mut() {
        transform.translation.x = layout.lane_x(label.lane);
        transform.translation.y = layout.judge_y - layout.label_offset_y;
        font.font_size = base_font.0 * layout.scale;
    }
}

pub(crate) fn apply_key_cap_layout(
    layout: Res<PlayfieldLayout>,
    mut caps: Query<
        (&keyboard_viz::KeyCap, &mut Sprite, &mut Transform),
        Without<keyboard_viz::KeyCapLabel>,
    >,
    mut labels: Query<
        (
            &keyboard_viz::KeyCap,
            &mut Transform,
            &ScaledFontSize,
            &mut TextFont,
        ),
        With<keyboard_viz::KeyCapLabel>,
    >,
) {
    if !layout.is_changed() {
        return;
    }

    keyboard_viz::apply_key_cap_sprites(&layout, &mut caps);
    keyboard_viz::apply_key_cap_labels(&layout, &mut labels);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_bar_w_insets_lane_width() {
        let layout = PlayfieldLayout::from_size(REF_WIDTH, REF_HEIGHT);
        assert!((layout.note_bar_w() - (layout.lane_w - 10.0 * layout.scale)).abs() < f32::EPSILON);
    }
}
