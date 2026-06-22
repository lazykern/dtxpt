use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};

use crate::app::markers::GameplayEntity;
use crate::gameplay::interp::RenderVisualClock;
use crate::gameplay::layout::PlayfieldLayout;
use dtxpt::chart::{BgaEvent, Chart};

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

pub(crate) fn update_bga_media(
    mut commands: Commands,
    chart: Res<Chart>,
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

    for (layer, bmp_id) in current_bga_layers(&chart.bga_events, render_clock.current) {
        if state
            .layer_entities
            .get(&layer)
            .is_some_and(|(current, _)| *current == bmp_id)
        {
            continue;
        }
        let Some(handle) = state.images.get(&bmp_id).cloned() else {
            continue;
        };
        if let Some((_, entity)) = state.layer_entities.remove(&layer) {
            commands.entity(entity).despawn();
        }
        let z = -4.9 + f32::from(layer) * 0.02;
        let entity = spawn_bga_sprite(&mut commands, handle, &layout, z, Color::WHITE);
        state.layer_entities.insert(layer, (bmp_id, entity));
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

fn current_bga_layers(events: &[BgaEvent], elapsed: f32) -> HashMap<u8, u32> {
    let mut layers = HashMap::new();
    for event in events {
        if event.time <= elapsed {
            layers.insert(event.layer, event.bmp_id);
        } else {
            break;
        }
    }
    layers
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
            },
            BgaEvent {
                time: 0.5,
                layer: 2,
                bmp_id: 2,
            },
            BgaEvent {
                time: 1.0,
                layer: 1,
                bmp_id: 3,
            },
        ];

        let layers = current_bga_layers(&events, 0.75);
        assert_eq!(layers.get(&1), Some(&1));
        assert_eq!(layers.get(&2), Some(&2));

        let layers = current_bga_layers(&events, 1.25);
        assert_eq!(layers.get(&1), Some(&3));
        assert_eq!(layers.get(&2), Some(&2));
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
