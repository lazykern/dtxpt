use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    SongSelect,
    LoadingSong,
    Playing,
    Result,
}

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum OverlayState {
    #[default]
    None,
    Settings,
}

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

pub fn is_paused(pause: &PauseState) -> bool {
    matches!(pause, PauseState::Paused)
}

pub fn overlay_closed(overlay: Res<State<OverlayState>>) -> bool {
    *overlay.get() == OverlayState::None
}
