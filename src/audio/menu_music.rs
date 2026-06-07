use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::AudioSource;
use kira::sound::static_sound::StaticSoundData;

use crate::audio::{AudioMix, MixKind, menu_fade_in_tween, menu_fade_out_tween};
use crate::current_song::CurrentSong;

#[derive(Resource)]
pub struct MenuMusicTrack;

const SELECT_DEBOUNCE: Duration = Duration::from_millis(150);

#[derive(Resource, Default)]
pub struct MenuBgmCache {
    sources: HashMap<PathBuf, Handle<AudioSource>>,
}

struct MenuBgmLoadTask {
    path: PathBuf,
    task: Task<Result<StaticSoundData, String>>,
}

#[derive(Resource, Default)]
pub struct MenuMusicState {
    playing_path: Option<PathBuf>,
    pending_path: Option<PathBuf>,
    pending_volume: i32,
    debounce: Timer,
    loading: Option<MenuBgmLoadTask>,
}

impl MenuMusicState {
    fn reset_debounce(&mut self, immediate: bool) {
        self.debounce = Timer::new(SELECT_DEBOUNCE, TimerMode::Once);
        if immediate {
            self.debounce.tick(SELECT_DEBOUNCE);
        }
    }

    fn clear_loading(&mut self) {
        self.loading = None;
    }
}

fn load_bgm_file(path: PathBuf) -> Result<StaticSoundData, String> {
    StaticSoundData::from_file(&path).map_err(|err| err.to_string())
}

fn play_cached_bgm(
    path: &PathBuf,
    volume: i32,
    cache: &MenuBgmCache,
    channel: &AudioChannel<MenuMusicTrack>,
    mix: &AudioMix,
) -> bool {
    let Some(source) = cache.sources.get(path) else {
        return false;
    };
    channel
        .play(source.clone())
        .with_volume(mix.volume_db(volume, MixKind::Bgm))
        .start_from(0.0)
        .loop_from(0.0)
        .fade_in(menu_fade_in_tween());
    true
}

pub fn update_menu_music(
    time: Res<Time>,
    current: Res<CurrentSong>,
    asset_server: Res<AssetServer>,
    channel: Res<AudioChannel<MenuMusicTrack>>,
    mix: Res<AudioMix>,
    mut cache: ResMut<MenuBgmCache>,
    mut state: ResMut<MenuMusicState>,
) {
    let pending_now = state.pending_path.clone();
    if let Some(loading) = state.loading.as_mut() {
        let loading_path = loading.path.clone();
        if pending_now.as_ref() != Some(&loading_path) {
            state.clear_loading();
        } else if let Some(result) = check_ready(&mut loading.task) {
            state.clear_loading();
            if pending_now.as_ref() == Some(&loading_path) {
                match result {
                    Ok(sound) => {
                        let source = asset_server.add(AudioSource { sound });
                        cache.sources.insert(loading_path.clone(), source);
                        state.playing_path = Some(loading_path.clone());
                        play_cached_bgm(
                            &loading_path,
                            state.pending_volume,
                            &cache,
                            &channel,
                            &mix,
                        );
                    }
                    Err(err) => {
                        warn!("failed to load menu BGM {}: {err}", loading_path.display());
                        state.playing_path = None;
                    }
                }
            }
            return;
        } else {
            return;
        }
    }

    let next_path = current.bgm_path.clone();
    let next_volume = current.bgm_volume;

    if state.pending_path != next_path || state.pending_volume != next_volume {
        state.pending_path = next_path;
        state.pending_volume = next_volume;
        let immediate = state.playing_path.is_none();
        state.reset_debounce(immediate);
        state.clear_loading();
        if state.playing_path.is_some() {
            channel.stop().fade_out(menu_fade_out_tween());
        }
    }

    state.debounce.tick(time.delta());
    if !state.debounce.is_finished() || state.playing_path == state.pending_path || state.loading.is_some()
    {
        return;
    }

    let Some(path) = state.pending_path.clone() else {
        state.playing_path = None;
        return;
    };

    if play_cached_bgm(&path, state.pending_volume, &cache, &channel, &mix) {
        state.playing_path = Some(path);
        return;
    }

    state.loading = Some(MenuBgmLoadTask {
        path: path.clone(),
        task: AsyncComputeTaskPool::get().spawn(async move { load_bgm_file(path) }),
    });
}

pub fn stop_menu_music(
    channel: Res<AudioChannel<MenuMusicTrack>>,
    mut state: ResMut<MenuMusicState>,
) {
    channel.stop().fade_out(menu_fade_out_tween());
    *state = MenuMusicState::default();
}
