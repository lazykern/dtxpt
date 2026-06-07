use bevy::prelude::*;

use crate::app::markers::*;
use crate::app::state::{PauseState, is_paused};
use crate::gameplay::clock::ChartClock;
use crate::gameplay::constants::*;
use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::run::RunState;
use dtxpt::chart::{Chart, Judgement, NoteState};
use dtxpt::input::lanes::LANES;

pub fn lane_receptor_color(lane: usize, strength: f32) -> Color {
    let s = strength.clamp(0.0, 1.0);
    let tint = LANES[lane].color.to_srgba();
    let base = LANE_RECEPTOR_BASE.to_srgba();
    let mix = 0.06 + 0.22 * s;
    let lift = 0.05 * s;
    Color::srgb(
        (base.red + tint.red * mix + lift).min(1.0),
        (base.green + tint.green * mix + lift).min(1.0),
        (base.blue + tint.blue * mix + lift).min(1.0),
    )
}

pub fn hit_burst_intensity(judgement: Judgement) -> f32 {
    match judgement {
        Judgement::Perfect => 1.0,
        Judgement::Great => 0.9,
        Judgement::Good => 0.75,
        Judgement::Poor => 0.55,
        Judgement::Miss => 0.4,
    }
}

pub fn spawn_hit_burst(
    commands: &mut Commands,
    layout: &PlayfieldLayout,
    lane: usize,
    judgement: Judgement,
    hit_y: f32,
) {
    let x = layout.lane_x(lane);
    let lane_color = LANES[lane].color;
    let intensity = hit_burst_intensity(judgement);
    let bar_w = layout.note_bar_w();
    let bar_h = layout.note_h;

    commands.spawn((
        Sprite::from_color(
            lane_color.with_alpha(0.9 * intensity),
            Vec2::new(bar_w, bar_h),
        ),
        Transform::from_xyz(x, hit_y, 3.0),
        HitBurst {
            timer: Timer::from_seconds(HIT_BURST_CORE_SECS, TimerMode::Once),
            kind: HitBurstKind::Core,
            lane_color,
            intensity,
            bar_w,
            bar_h,
        },
        GameplayEntity,
    ));
    commands.spawn((
        Sprite::from_color(
            lane_color.with_alpha(0.3 * intensity),
            Vec2::new(bar_w, bar_h * 1.6),
        ),
        Transform::from_xyz(x, hit_y, 2.5),
        HitBurst {
            timer: Timer::from_seconds(HIT_BURST_GLOW_SECS, TimerMode::Once),
            kind: HitBurstKind::Glow,
            lane_color,
            intensity,
            bar_w,
            bar_h,
        },
        GameplayEntity,
    ));
}

pub(crate) fn update_metronome_lines(
    mut commands: Commands,
    chart: Res<Chart>,
    run: Res<RunState>,
    clock: Res<ChartClock>,
    layout: Res<PlayfieldLayout>,
    mut lines: Query<(Entity, &MetronomeLineVisual, &mut Transform, &mut Sprite)>,
) {
    for (entity, visual, mut transform, mut sprite) in lines.iter_mut() {
        let beat = &chart.metronome_beats[visual.beat_index];
        transform.translation.y = layout.note_y(beat.time, clock.visual_elapsed, run.lane_speed);
        let (height, base_alpha) = if beat.downbeat {
            (layout.metronome_line_height * 1.4, 0.55)
        } else {
            (layout.metronome_line_height, 0.28)
        };
        sprite.custom_size = Some(Vec2::new(layout.judge_line_width, height));
        let fade =
            ((transform.translation.y - layout.judge_y) / layout.note_fade_span).clamp(0.15, 1.0);
        sprite.color = sprite.color.with_alpha(base_alpha * fade);
        if transform.translation.y < layout.judge_y - layout.note_fade_span {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn update_note_visuals(
    mut commands: Commands,
    chart: Res<Chart>,
    run: Res<RunState>,
    clock: Res<ChartClock>,
    layout: Res<PlayfieldLayout>,
    mut notes: Query<(Entity, &NoteVisual, &mut Transform, &mut Sprite)>,
) {
    for (entity, visual, mut transform, mut sprite) in notes.iter_mut() {
        let note = &chart.notes[visual.note_index];
        match note.state {
            NoteState::Pending => {
                transform.translation.x = layout.lane_x(note.lane);
                transform.translation.y =
                    layout.note_y(note.time, clock.visual_elapsed, run.lane_speed);
                sprite.custom_size = Some(Vec2::new(layout.note_bar_w(), layout.note_h));
                let fade = ((transform.translation.y - layout.judge_y) / layout.note_fade_span)
                    .clamp(0.25, 1.0);
                sprite.color = LANES[note.lane].color.with_alpha(fade);
            }
            NoteState::Hit(_) | NoteState::Missed | NoteState::Skipped => {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub(crate) fn update_lane_receptor_flashes(
    time: Res<Time>,
    pause_state: Res<State<PauseState>>,
    mut receptors: Query<(&LaneReceptor, &mut Sprite, &mut LaneReceptorFlash)>,
) {
    if is_paused(pause_state.get()) {
        return;
    }

    for (receptor, mut sprite, mut flash) in receptors.iter_mut() {
        if !flash.timer.is_finished() {
            flash.timer.tick(time.delta());
        }
        let strength = if flash.timer.is_finished() {
            0.0
        } else {
            1.0 - flash.timer.fraction()
        };
        sprite.color = lane_receptor_color(receptor.lane, strength);
    }
}

pub(crate) fn update_hit_bursts(
    time: Res<Time>,
    pause_state: Res<State<PauseState>>,
    mut commands: Commands,
    mut bursts: Query<(Entity, &mut HitBurst, &mut Sprite)>,
) {
    if is_paused(pause_state.get()) {
        return;
    }

    for (entity, mut burst, mut sprite) in bursts.iter_mut() {
        burst.timer.tick(time.delta());
        let t = burst.timer.fraction();
        let fade = 1.0 - t;

        match burst.kind {
            HitBurstKind::Core => {
                sprite.color = burst.lane_color.with_alpha(0.9 * burst.intensity * fade);
                sprite.custom_size = Some(Vec2::new(burst.bar_w, burst.bar_h));
            }
            HitBurstKind::Glow => {
                let expand = 1.0 + t * 0.4;
                sprite.color = burst
                    .lane_color
                    .with_alpha(0.3 * burst.intensity * fade * fade);
                sprite.custom_size =
                    Some(Vec2::new(burst.bar_w * expand, burst.bar_h * 1.6 * expand));
            }
        }

        if burst.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
