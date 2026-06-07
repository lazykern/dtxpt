use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::config::GameConfig;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MixKind {
    Bgm,
    Drums,
}

#[derive(Resource, Clone)]
pub struct AudioMix {
    pub master: f32,
    pub bgm: f32,
    pub drums: f32,
}

impl AudioMix {
    pub fn from_config(config: &GameConfig) -> Self {
        Self {
            master: config.master_volume as f32,
            bgm: config.bgm_volume as f32,
            drums: config.drum_volume as f32,
        }
    }

    pub fn volume_db(&self, dtx_volume: i32, kind: MixKind) -> f32 {
        let channel = match kind {
            MixKind::Bgm => self.bgm,
            MixKind::Drums => self.drums,
        };
        linear_gain_to_db(
            dtx_linear(dtx_volume) * self.master.clamp(0.0, 1.0) * channel.clamp(0.0, 1.0),
        )
    }

    pub fn master_db(&self) -> f32 {
        linear_gain_to_db(self.master.clamp(0.0, 1.0))
    }
}

pub fn instant_audio_tween() -> AudioTween {
    AudioTween::linear(std::time::Duration::from_millis(0))
}

pub fn menu_fade_out_tween() -> AudioTween {
    AudioTween::linear(std::time::Duration::from_millis(150))
}

pub fn menu_fade_in_tween() -> AudioTween {
    AudioTween::linear(std::time::Duration::from_millis(220))
}

pub(crate) fn dtx_linear(volume: i32) -> f32 {
    if volume <= 0 {
        0.0
    } else {
        (volume as f32 / 100.0).clamp(0.0, 1.0)
    }
}

pub(crate) fn linear_gain_to_db(gain: f32) -> f32 {
    if gain <= 0.0 {
        -60.0
    } else {
        20.0 * gain.log10()
    }
}

pub(crate) fn dtx_pan_to_kira(pan: i32) -> f32 {
    (pan as f32 / 100.0).clamp(-1.0, 1.0)
}
