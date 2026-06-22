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
        && config.guitar_offset == run.guitar_offset
        && config.bass_offset == run.bass_offset
        && config.song_playback_rate == run.song_playback_rate
        && config.practice_song_select == run.practice
        && config.metronome_sound == run.metronome_sound
        && config.lp_muting == run.lp_muting
        && config.drum_hit_sound == run.drum_hit_sound
        && config.pedal_lag_time_ms == run.pedal_lag_time_ms
        && config.cymbal_free == run.cymbal_free
        && config.hit_sound_priority_hh == run.hit_sound_priority_hh
        && config.hit_sound_priority_ft == run.hit_sound_priority_ft
        && config.hit_sound_priority_cy == run.hit_sound_priority_cy
        && config.hit_sound_priority_lp == run.hit_sound_priority_lp
        && config.show_debug_hud == run.show_debug_hud
    {
        return;
    }

    config.master_volume = master_volume;
    config.bgm_volume = bgm_volume;
    config.drum_volume = drum_volume;
    config.lane_speed = run.lane_speed;
    config.timing_offset = run.timing_offset;
    config.guitar_offset = run.guitar_offset;
    config.bass_offset = run.bass_offset;
    config.song_playback_rate = run.song_playback_rate;
    config.practice_song_select = run.practice;
    config.metronome_sound = run.metronome_sound;
    config.lp_muting = run.lp_muting;
    config.drum_hit_sound = run.drum_hit_sound;
    config.pedal_lag_time_ms = run.pedal_lag_time_ms;
    config.cymbal_free = run.cymbal_free;
    config.hit_sound_priority_hh = run.hit_sound_priority_hh;
    config.hit_sound_priority_ft = run.hit_sound_priority_ft;
    config.hit_sound_priority_cy = run.hit_sound_priority_cy;
    config.hit_sound_priority_lp = run.hit_sound_priority_lp;
    config.show_debug_hud = run.show_debug_hud;

    if let Err(err) = save_game_config(&config) {
        warn!("failed to save config: {err}");
    }
}
