use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub(crate) enum HitBurstKind {
    Core,
    Glow,
}

#[derive(Component)]
pub(crate) struct HitBurst {
    pub timer: Timer,
    pub kind: HitBurstKind,
    pub lane_color: Color,
    pub intensity: f32,
    pub bar_w: f32,
    pub bar_h: f32,
}

#[derive(Component)]
pub(crate) struct LaneReceptor {
    pub lane: usize,
}

#[derive(Component)]
pub(crate) struct LaneReceptorFlash {
    pub timer: Timer,
}

#[derive(Component)]
pub(crate) struct NoteVisual {
    pub note_index: usize,
}

#[derive(Component)]
pub(crate) struct MetronomeLineVisual {
    pub beat_index: usize,
}

#[derive(Component)]
pub(crate) struct GameplayEntity;

#[derive(Component)]
pub(crate) struct MenuBackground;

#[derive(Component)]
pub(crate) struct MainMenuScreen;

#[derive(Component)]
pub(crate) struct MainMenuAction(pub MainMenuChoice);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MainMenuChoice {
    SongSelect,
    Settings,
    Quit,
}

#[derive(Component)]
pub(crate) struct SongSelectList;

#[derive(Component)]
pub(crate) struct SongSelectMeta;

#[derive(Component)]
pub(crate) struct SongSelectCard {
    pub entry_index: usize,
}

#[derive(Component)]
pub(crate) struct SettingsRowMarker {
    pub index: usize,
}

#[derive(Component)]
pub(crate) struct GameplayHudRoot;

#[derive(Component)]
pub(crate) struct GameplayHudScore;

#[derive(Component)]
pub(crate) struct GameplayHudCombo;

#[derive(Component)]
pub(crate) struct GameplayHudAccuracy;

#[derive(Component)]
pub(crate) struct GameplayHudGauge;

#[derive(Component)]
pub(crate) struct GameplayHudGaugeFill;

#[derive(Component)]
pub(crate) struct GameplayHudCounters;

#[derive(Component)]
pub(crate) struct GameplayHudDebug;

#[derive(Component)]
pub(crate) struct GameplayHudDebugText;

#[derive(Component)]
pub(crate) struct GameplayHudJudgement;

#[derive(Component)]
pub(crate) struct SongSelectScreen;

#[derive(Component)]
pub(crate) struct SongSelectPreviewFrame;

#[derive(Component)]
pub(crate) struct SongSelectPreviewImage;

#[derive(Component)]
pub(crate) struct LoadingScreen;

#[derive(Component)]
pub(crate) struct ResultScreen;

#[derive(Component)]
pub(crate) struct SettingsOverlayScreen;

#[derive(Component)]
pub(crate) struct PauseOverlayScreen;

#[derive(Component)]
pub(crate) struct PauseActionButton(pub PauseAction);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PauseAction {
    Resume,
    Retry,
    SongSelect,
}

#[derive(Component)]
pub(crate) struct ResultActionButton(pub ResultAction);

#[derive(Clone, Copy)]
pub(crate) enum ResultAction {
    Retry,
    SongSelect,
}

#[derive(Component)]
pub(crate) struct JudgementText;

#[derive(Component)]
pub(crate) struct GaugeBarTrack;

#[derive(Component)]
pub(crate) struct GaugeBarFill;

#[derive(Component)]
pub(crate) struct PlayfieldBackboard;

#[derive(Component)]
pub(crate) struct LaneColumn {
    pub lane: usize,
}

#[derive(Component)]
pub(crate) struct LaneLabel {
    pub lane: usize,
}

#[derive(Component)]
pub(crate) struct JudgeLine;

#[derive(Component)]
pub(crate) struct ScaledFontSize(pub f32);
