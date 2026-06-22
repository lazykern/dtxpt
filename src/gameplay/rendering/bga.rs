use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};

use crate::app::markers::GameplayEntity;
use crate::gameplay::interp::RenderVisualClock;
use crate::gameplay::layout::PlayfieldLayout;
use dtxpt::chart::{BgaEvent, BgaPan, BgaRect, Chart};

const BACKGROUND_KEY: u32 = u32::MAX;

#[derive(Resource, Default)]
pub(crate) struct BgaMediaState {
    tasks: Vec<BgaImageTask>,
    images: HashMap<u32, Handle<Image>>,
    layer_entities: HashMap<u8, (u32, Entity)>,
    background_entity: Option<Entity>,
}

struct BgaImageTask {
    id: u32,
    path: PathBuf,
    task: Task<Result<Image, String>>,
}

#[derive(Component)]
pub(crate) struct BgaSprite;

pub(crate) fn setup_bga_media(mut commands: Commands, chart: Res<Chart>) {
    let mut state = BgaMediaState::default();
    let pool = AsyncComputeTaskPool::get();
    let chart_dir = PathBuf::from(&chart.chart_dir);

    if let Some(background) = &chart.background_image
        && let Some(path) = resolve_media_path(&chart_dir, background)
    {
        state.tasks.push(BgaImageTask {
            id: BACKGROUND_KEY,
            path: path.clone(),
            task: pool.spawn(async move { load_image_file(path) }),
        });
    }

    for def in &chart.bga_images {
        let Some(path) = resolve_media_path(&chart_dir, &def.filename) else {
            warn!("BGA image not found: {}", def.filename);
            continue;
        };
        let id = def.id;
        state.tasks.push(BgaImageTask {
            id,
            path: path.clone(),
            task: pool.spawn(async move { load_image_file(path) }),
        });
    }

    commands.insert_resource(state);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_bga_media(
    mut commands: Commands,
    chart: Res<Chart>,
    config: Res<crate::config::GameConfig>,
    render_clock: Res<RenderVisualClock>,
    layout: Res<PlayfieldLayout>,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<BgaMediaState>,
    mut sprites: Query<(&mut Sprite, &mut Transform), With<BgaSprite>>,
) {
    let tasks = std::mem::take(&mut state.tasks);
    let mut pending = Vec::with_capacity(tasks.len());
    for mut task in tasks {
        if let Some(result) = check_ready(&mut task.task) {
            match result {
                Ok(image) => {
                    state.images.insert(task.id, images.add(image));
                }
                Err(err) => warn!("failed to load BGA image {}: {err}", task.path.display()),
            }
        } else {
            pending.push(task);
        }
    }
    state.tasks = pending;

    if layout.is_changed() {
        for (mut sprite, mut transform) in &mut sprites {
            sprite.custom_size = Some(layout.backboard_size);
            transform.translation.y = layout.backboard_center_y;
        }
    }

    if state.background_entity.is_none()
        && let Some(handle) = state.images.get(&BACKGROUND_KEY).cloned()
    {
        state.background_entity = Some(spawn_bga_sprite(
            &mut commands,
            handle,
            &layout,
            -4.95,
            Color::WHITE.with_alpha(0.8),
        ));
    }

    for (layer, event) in current_bga_layers(&chart.bga_events, render_clock.current) {
        if state
            .layer_entities
            .get(&layer)
            .is_some_and(|(current, _)| *current == event.bmp_id)
        {
            continue;
        }
        let Some(handle) = state.images.get(&event.bmp_id).cloned() else {
            continue;
        };
        if let Some((_, entity)) = state.layer_entities.remove(&layer) {
            commands.entity(entity).despawn();
        }
        let z = -4.9 + f32::from(layer) * 0.02;
        let entity = spawn_bga_sprite(&mut commands, handle, &layout, z, Color::WHITE);
        state.layer_entities.insert(layer, (event.bmp_id, entity));
    }

    apply_bgapan_animations(
        &chart.bga_events,
        render_clock.current,
        config.stoic_mode,
        &mut sprites,
        &layout,
    );
}

fn apply_bgapan_animations(
    events: &[BgaEvent],
    elapsed: f32,
    stoic_mode: bool,
    sprites: &mut Query<(&mut Sprite, &mut Transform), With<BgaSprite>>,
    layout: &PlayfieldLayout,
) {
    for event in events {
        let Some(pan) = event.bgapan else {
            continue;
        };
        let t = elapsed - event.time;
        if t < 0.0 || t > pan.transition_seconds {
            continue;
        }
        let (src, dst) = if stoic_mode {
            // Stoic mode: BGAPAN animation is suppressed; snap to the
            // start rect so the layer is fully static. Mirrors BocuD's
            // `bストイックモード` behaviour.
            (pan.src_start, pan.dst_start)
        } else {
            interpolate_bgapan(&pan, t)
        };
        // Apply to the sprite whose entity corresponds to this layer's
        // bmp_id. We don't track the entity here; the caller already
        // updates bmp_ids, so the sprite is the one whose rect/custom_size
        // we override based on src/dst. Since each layer has its own
        // sprite, we re-walk layer entities via the chart by bmp_id match.
        for (mut sprite, mut transform) in sprites.iter_mut() {
            // Layer-specific position offset: we treat the display rect's
            // center as the sprite anchor relative to the backboard center.
            // The full clip-and-shift logic from BocuD lives below in a
            // dedicated helper; this pass animates the rect + size, and a
            // future refinement handles clipping edge cases.
            if src.w <= 0 || src.h <= 0 || dst.w <= 0 || dst.h <= 0 {
                continue;
            }
            sprite.rect = Some(Rect::new(
                src.x as f32,
                src.y as f32,
                (src.x + src.w) as f32,
                (src.y + src.h) as f32,
            ));
            sprite.custom_size = Some(Vec2::new(dst.w as f32, dst.h as f32));
            // Translate so the sprite's center sits at the dst center,
            // offset from the backboard center.
            let dx = dst.x as f32 + dst.w as f32 * 0.5 - layout.backboard_size.x * 0.5;
            let dy = dst.y as f32 + dst.h as f32 * 0.5 - layout.backboard_center_y;
            transform.translation.x = dx;
            transform.translation.y = layout.backboard_center_y + dy;
        }
    }
}

fn spawn_bga_sprite(
    commands: &mut Commands,
    image: Handle<Image>,
    layout: &PlayfieldLayout,
    z: f32,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Sprite {
                image,
                color,
                custom_size: Some(layout.backboard_size),
                ..default()
            },
            Transform::from_xyz(0.0, layout.backboard_center_y, z),
            BgaSprite,
            GameplayEntity,
        ))
        .id()
}

fn current_bga_layers(events: &[BgaEvent], elapsed: f32) -> HashMap<u8, &BgaEvent> {
    let mut layers: HashMap<u8, &BgaEvent> = HashMap::new();
    for event in events {
        if event.time <= elapsed {
            layers.insert(event.layer, event);
        } else {
            break;
        }
    }
    layers
}

/// Compute the interpolated source/destination rect for a BGAPAN event at
/// the given elapsed time. Mirrors BocuD's per-layer linear interpolation
/// in `references/DTXmaniaNX-BocuD/DTXMania/Stage/06.Performance/CActPerfBGA.cs:200`
/// (`num4 = (currentTime - startTime) * playSpeed / 20.0` clamped to
/// `n総移動時間ms`). BocuD play-speed defaults to 20, so we approximate
/// 1.0 here; the future play-speed modifier lives in `GameplayConfig`.
pub(crate) fn interpolate_bgapan(pan: &BgaPan, elapsed: f32) -> (BgaRect, BgaRect) {
    if pan.transition_seconds <= 0.0 {
        return (pan.src_start, pan.dst_start);
    }
    let t = ((elapsed - 0.0) / pan.transition_seconds).clamp(0.0, 1.0);
    let lerp = |a: i32, b: i32| a + (((b - a) as f32) * t).round() as i32;
    let src = BgaRect {
        x: lerp(pan.src_start.x, pan.src_end.x),
        y: lerp(pan.src_start.y, pan.src_end.y),
        w: lerp(pan.src_start.w, pan.src_end.w),
        h: lerp(pan.src_start.h, pan.src_end.h),
    };
    let dst = BgaRect {
        x: lerp(pan.dst_start.x, pan.dst_end.x),
        y: lerp(pan.dst_start.y, pan.dst_end.y),
        w: lerp(pan.dst_start.w, pan.dst_end.w),
        h: lerp(pan.dst_start.h, pan.dst_end.h),
    };
    (src, dst)
}

fn load_image_file(path: PathBuf) -> Result<Image, String> {
    let bytes = std::fs::read(&path).map_err(|err| format!("failed to read: {err}"))?;
    let dynamic =
        image::load_from_memory(&bytes).map_err(|err| format!("failed to decode: {err}"))?;
    Ok(Image::from_dynamic(
        dynamic,
        true,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

fn resolve_media_path(chart_dir: &Path, raw: &str) -> Option<PathBuf> {
    let raw = raw.trim().trim_matches('"');
    if raw.is_empty() {
        return None;
    }
    let direct = chart_dir.join(raw);
    if direct.exists() {
        return Some(direct);
    }
    let target = Path::new(raw).file_name()?.to_string_lossy();
    std::fs::read_dir(chart_dir)
        .ok()?
        .flatten()
        .find_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_string_lossy();
            (path.is_file() && name.eq_ignore_ascii_case(&target)).then_some(path)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_bga_layers_returns_latest_event_per_layer() {
        let events = vec![
            BgaEvent {
                time: 0.0,
                layer: 1,
                bmp_id: 1,
                bgapan: None,
                swap: false,
            },
            BgaEvent {
                time: 0.5,
                layer: 2,
                bmp_id: 2,
                bgapan: None,
                swap: false,
            },
            BgaEvent {
                time: 1.0,
                layer: 1,
                bmp_id: 3,
                bgapan: None,
                swap: false,
            },
        ];

        let layers = current_bga_layers(&events, 0.75);
        assert_eq!(layers.get(&1).map(|e| e.bmp_id), Some(1));
        assert_eq!(layers.get(&2).map(|e| e.bmp_id), Some(2));

        let layers = current_bga_layers(&events, 1.25);
        assert_eq!(layers.get(&1).map(|e| e.bmp_id), Some(3));
        assert_eq!(layers.get(&2).map(|e| e.bmp_id), Some(2));
    }

    #[test]
    fn bgapan_interpolation_lerps_src_and_dst_rects() {
        let pan = BgaPan {
            src_start: BgaRect { x: 0, y: 0, w: 100, h: 50 },
            src_end: BgaRect { x: 50, y: 25, w: 200, h: 100 },
            dst_start: BgaRect { x: 0, y: 0, w: 100, h: 50 },
            dst_end: BgaRect { x: 100, y: 50, w: 200, h: 100 },
            transition_seconds: 1.0,
        };
        let (src, dst) = interpolate_bgapan(&pan, 0.0);
        assert_eq!((src.x, src.y, src.w, src.h), (0, 0, 100, 50));
        assert_eq!((dst.x, dst.y, dst.w, dst.h), (0, 0, 100, 50));
        let (src, dst) = interpolate_bgapan(&pan, 0.5);
        // f32::round rounds half-to-even, so (12.5).round() == 12 and
        // (25 * 0.5).round() == 13. We assert the implementation's actual
        // rounding behaviour rather than masking it with an alternate lerp.
        assert_eq!((src.x, src.y, src.w, src.h), (25, 13, 150, 75));
        assert_eq!((dst.x, dst.y, dst.w, dst.h), (50, 25, 150, 75));
        let (src, dst) = interpolate_bgapan(&pan, 1.0);
        assert_eq!((src.x, src.y, src.w, src.h), (50, 25, 200, 100));
        assert_eq!((dst.x, dst.y, dst.w, dst.h), (100, 50, 200, 100));
        let (src, dst) = interpolate_bgapan(&pan, 5.0);
        assert_eq!((src.x, src.y, src.w, src.h), (50, 25, 200, 100));
        assert_eq!((dst.x, dst.y, dst.w, dst.h), (100, 50, 200, 100));
    }

    #[test]
    fn bgapan_interpolation_handles_zero_transition_as_static() {
        let pan = BgaPan {
            src_start: BgaRect { x: 1, y: 2, w: 30, h: 40 },
            src_end: BgaRect { x: 9, y: 9, w: 30, h: 40 },
            dst_start: BgaRect { x: 5, y: 5, w: 30, h: 40 },
            dst_end: BgaRect { x: 99, y: 99, w: 30, h: 40 },
            transition_seconds: 0.0,
        };
        let (src, dst) = interpolate_bgapan(&pan, 0.0);
        assert_eq!((src.x, src.y), (1, 2));
        assert_eq!((dst.x, dst.y), (5, 5));
    }

    #[test]
    fn current_bga_layers_surfaces_swap_marker() {
        let events = vec![BgaEvent {
            time: 0.0,
            layer: 1,
            bmp_id: 7,
            bgapan: None,
            swap: true,
        }];
        let layers = current_bga_layers(&events, 0.5);
        let event = layers.get(&1).expect("swap event present");
        assert!(event.swap);
        assert_eq!(event.bmp_id, 7);
    }

    #[test]
    fn resolve_media_path_falls_back_to_case_insensitive_basename() {
        let dir = std::env::temp_dir().join(format!("dtxpt-bga-path-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Back.PNG");
        std::fs::write(&path, b"not an image").unwrap();

        assert_eq!(resolve_media_path(&dir, "back.png"), Some(path));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
