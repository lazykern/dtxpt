use bevy::prelude::*;

use super::animation::{update_button_interactions, update_screen_fade_in};
use super::fonts::UiFonts;
use super::skin::ActiveSkin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiFonts>()
            .init_resource::<ActiveSkin>()
            .add_systems(Update, (update_button_interactions, update_screen_fade_in));
    }
}
