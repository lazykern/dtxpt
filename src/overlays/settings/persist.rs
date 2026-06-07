use bevy::prelude::*;

use crate::audio::AudioMix;
use crate::config::{GameConfig, save_game_config};
use crate::gameplay::run::RunState;

pub fn persist_runtime_config(
    mut config: ResMut<GameConfig>,
    run: Res<RunState>,
    mix: Res<AudioMix>,
) {
    let master_volume = mix.master as f64;
    let bgm_volume = mix.bgm as f64;
    let drum_volume = mix.drums as f64;

    if config.master_volume == master_volume
        && config.bgm_volume == bgm_volume
        && config.drum_volume == drum_volume
        && config.lane_speed == run.lane_speed
        && config.timing_offset == run.timing_offset
        && config.song_playback_rate == run.song_playback_rate
        && config.play_mode == run.play_mode
        && config.metronome_sound == run.metronome_sound
        && config.show_debug_hud == run.show_debug_hud
    {
        return;
    }

    config.master_volume = master_volume;
    config.bgm_volume = bgm_volume;
    config.drum_volume = drum_volume;
    config.lane_speed = run.lane_speed;
    config.timing_offset = run.timing_offset;
    config.song_playback_rate = run.song_playback_rate;
    config.play_mode = run.play_mode;
    config.metronome_sound = run.metronome_sound;
    config.show_debug_hud = run.show_debug_hud;

    if let Err(err) = save_game_config(&config) {
        warn!("failed to save config: {err}");
    }
}
