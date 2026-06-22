use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    SongSelect,
    LoadingSong,
    Playing,
    /// Interstitial screen between gameplay and Result, showing a
    /// Stage Clear! / Stage Failed! banner and playing the
    /// corresponding SFX. Auto-advances to `Result` after a short
    /// delay. Skipped when `bSTAGEFAILEDEnabled` is off and the player
    /// cleared, or when no SFX is configured.
    /// BocuD ref: `CActPerformanceStageClear` /
    /// `CActPerformanceStageFailed` (`references/DTXmaniaNX-BocuD/DTXMania/Stage/06.Performance/`).
    StageClear,
    StageFailed,
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

pub fn initial_app_state(compact_mode: bool) -> AppState {
    if compact_mode {
        AppState::SongSelect
    } else {
        AppState::MainMenu
    }
}

/// Per-instrument gameplay axis. Derived from `AppState::Playing` so
/// it only exists during gameplay. Used to route per-instrument
/// gameplay systems (Drum/Guitar/Bass) to the right slice of
/// `Chart` (notes / guitar_notes / bass_notes). See
/// `docs/plans/bocu-d-port-architecture.md` §B-4.
#[derive(SubStates, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[source(AppState = AppState::Playing)]
pub enum PerfPart {
    #[default]
    Drums,
    Guitar,
    Bass,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_mode_starts_at_song_select() {
        assert_eq!(initial_app_state(true), AppState::SongSelect);
        assert_eq!(initial_app_state(false), AppState::MainMenu);
    }
}
