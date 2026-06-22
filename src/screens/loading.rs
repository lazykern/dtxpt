use bevy::prelude::*;
use dtxpt::chart::{
    Chart, ChartTiming, ChipPlayTimeComputeMode, load_chart_from_path_with_compute_mode,
};

use crate::app::markers::LoadingScreen;
use crate::app::state::AppState;
use crate::audio::{ActiveSounds, AudioFrame, GameRng, MetronomeActive};
use crate::config::GameConfig;
use crate::gameplay::SelectedChartPath;
use crate::gameplay::clock::ChartClock;
use crate::gameplay::run::RunState;
use crate::ui::fonts::{UiFonts, text_font};
use crate::ui::palette::*;
use crate::ui::theme::*;
use crate::ui::widgets::*;
use dtxpt::util::background_task::{BackgroundPoll, BackgroundTask};

type ChartLoadResult = Result<(Chart, ChartTiming), String>;

#[derive(Resource, Default)]
pub struct ChartLoad {
    pub loading: bool,
    task: BackgroundTask<ChartLoadResult>,
}

impl ChartLoad {
    pub fn start(&mut self, path: String, chip_play_time_compute_mode: ChipPlayTimeComputeMode) {
        self.task.start(move || {
            load_chart_from_path_with_compute_mode(&path, chip_play_time_compute_mode)
                .map_err(|err| err.to_string())
        });
        self.loading = true;
    }

    pub fn poll(&mut self) -> Option<ChartLoadResult> {
        match self.task.poll() {
            BackgroundPoll::Ready(value) => {
                self.loading = false;
                Some(value)
            }
            BackgroundPoll::Disconnected => {
                self.loading = false;
                Some(Err("chart load thread disconnected".into()))
            }
            BackgroundPoll::Pending => None,
        }
    }

    pub fn reset(&mut self) {
        self.loading = false;
        self.task.reset();
    }
}

pub fn setup_loading_screen(
    mut commands: Commands,
    fonts: Res<UiFonts>,
    selected: Res<SelectedChartPath>,
) {
    commands.spawn((
        screen_root(),
        LoadingScreen,
        children![(
            centered_column(SPACING_LG),
            children![
                (
                    Text::new("Loading"),
                    text_font(&fonts, FONT_HEADING),
                    TextColor(WARNING),
                ),
                progress_bar_bundle(320.0, 8.0, 0.35, ACCENT),
                (
                    Text::new(selected.0.clone()),
                    text_font(&fonts, FONT_CAPTION),
                    TextColor(TEXT_SECONDARY),
                ),
            ],
        )],
    ));
}

pub fn start_chart_load(
    selected: Res<SelectedChartPath>,
    config: Res<GameConfig>,
    mut load: ResMut<ChartLoad>,
) {
    load.start(selected.0.clone(), config.chip_play_time_compute_mode);
}

pub fn poll_chart_load(
    mut commands: Commands,
    selected: Res<SelectedChartPath>,
    config: Res<GameConfig>,
    mut load: ResMut<ChartLoad>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(result) = load.poll() else {
        return;
    };

    match result {
        Ok((chart, timing)) => {
            info!("loaded selected chart: {}", selected.0);
            commands.insert_resource(chart);
            commands.insert_resource(timing);
            commands.insert_resource(RunState::from_config(&config));
            commands.insert_resource(ChartClock::default());
            commands.insert_resource(ActiveSounds::default());
            commands.insert_resource(MetronomeActive::default());
            commands.insert_resource(GameRng::default());
            commands.insert_resource(AudioFrame::default());
            next_state.set(AppState::Playing);
        }
        Err(err) => {
            warn!("failed to load selected chart: {err}");
            next_state.set(AppState::MainMenu);
        }
    }
}

pub fn reset_chart_load(mut load: ResMut<ChartLoad>) {
    load.reset();
}
