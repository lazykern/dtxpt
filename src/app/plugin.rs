use bevy::prelude::*;
use bevy_brp_extras::BrpExtrasPlugin;
use bevy_framepace::{FramepacePlugin, FramepaceSettings};

use bevy::window::{MonitorSelection, Window, WindowMode, WindowPlugin};
use bevy_kira_audio::prelude::*;
use dtxpt::input::{InputBindings, midi};
use dtxpt::song_library;

use crate::app::state::{OverlayState, PauseState, initial_app_state};
use crate::audio::AudioFrame;
use crate::audio::{ActiveSounds, AudioMix, GameRng, MetronomeActive};
use crate::audio::{MenuBgmCache, MenuMusicState, MenuMusicTrack};
use crate::config::{initial_chart_path, library_cache_path, load_game_config};
use crate::current_song::{
    CurrentSong, align_library_to_current_song, enrich_current_song_from_library,
};
use crate::gameplay::clock::{ChartClock, RenderStats};

use crate::gameplay::layout::PlayfieldLayout;
use crate::gameplay::plugin::GameplayPlugin;
use crate::gameplay::run::{RunState, SelectedChartPath};
use crate::overlays::plugin::OverlaysPlugin;
use crate::overlays::settings::SettingsOverlay;
use crate::persistence::load_score_store;
use crate::screens::plugin::ScreensPlugin;
use crate::screens::song_select::SongPreviewImage;
use crate::ui::UiPlugin;
use crate::ui::theme::{REF_HEIGHT, REF_WIDTH};
use dtxpt::chart::resolve_chart_bgm;

pub struct DtxptPlugin;

impl Plugin for DtxptPlugin {
    fn build(&self, app: &mut App) {
        let config = load_game_config();
        let input_bindings = InputBindings::from_config(&config.bindings);
        let audio_mix = AudioMix::from_config(&config);
        let fps_cap = config.fps_cap;
        let initial_app_state = initial_app_state(config.compact_mode);
        let score_store = load_score_store();
        let chart_path = initial_chart_path(&config);
        let (mut song_library, song_scan) =
            song_library::start_library_scan(&config.chart_root, &library_cache_path());
        align_library_to_current_song(
            &mut song_library,
            &CurrentSong::from_path_stub(&chart_path),
            &config.preferred_difficulty,
        );
        let mut current_song = CurrentSong::from_library(&song_library)
            .unwrap_or_else(|| CurrentSong::from_path_stub(&chart_path));
        enrich_current_song_from_library(&mut current_song, &mut song_library);
        if current_song.bgm_path.is_none()
            && !chart_path.is_empty()
            && let Some(bgm) = resolve_chart_bgm(std::path::Path::new(&chart_path))
        {
            current_song.bgm_path = Some(bgm.path);
            current_song.bgm_volume = bgm.volume;
        }
        let selected_chart = SelectedChartPath(current_song.chart_path.clone());

        app.insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.12)))
            .insert_resource(fps_cap.winit_settings())
            .insert_resource(FramepaceSettings {
                limiter: fps_cap.limiter(),
            })
            .insert_resource(config)
            .insert_resource(input_bindings)
            .insert_resource(audio_mix)
            .insert_resource(score_store)
            .init_resource::<MenuBgmCache>()
            .init_resource::<MenuMusicState>()
            .init_resource::<SongPreviewImage>()
            .insert_resource(current_song)
            .insert_resource(selected_chart)
            .insert_resource(song_library)
            .insert_resource(song_scan)
            .init_resource::<SettingsOverlay>()
            .init_resource::<RunState>()
            .init_resource::<ChartClock>()
            .init_resource::<RenderStats>()
            .init_resource::<PlayfieldLayout>()
            .init_resource::<ActiveSounds>()
            .init_resource::<MetronomeActive>()
            .init_resource::<GameRng>()
            .init_resource::<AudioFrame>()
            .add_plugins((
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some({
                        let windowed = std::env::var("DTXPT_WINDOWED").is_ok();
                        Window {
                            title: "dtxpt".into(),
                            mode: if windowed {
                                WindowMode::Windowed
                            } else {
                                WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                            },
                            resolution: if windowed {
                                (1280, 780).into()
                            } else {
                                (REF_WIDTH as u32, REF_HEIGHT as u32).into()
                            },
                            present_mode: fps_cap.present_mode(),
                            ..default()
                        }
                    }),
                    ..default()
                }),
                AudioPlugin,
            ))
            .add_audio_channel::<MenuMusicTrack>()
            .insert_state(initial_app_state)
            .init_state::<OverlayState>()
            .init_state::<PauseState>()
            .add_sub_state::<crate::app::state::PerfPart>();

        midi::plugin(app);
        app.add_plugins((
            UiPlugin,
            ScreensPlugin,
            OverlaysPlugin,
            GameplayPlugin,
            FramepacePlugin,
            BrpExtrasPlugin::default(),
        ));
    }
}
