use std::path::Path;

use bevy::prelude::*;

use crate::util::background_task::{BackgroundPoll, BackgroundTask};

use super::cache::load_cached_library;
use super::model::{SongLibrary, SongSortMode};
use super::scanner::scan_song_library;

#[derive(Resource, Debug)]
pub struct SongLibraryScan {
    pub scanning: bool,
    task: BackgroundTask<SongLibrary>,
}

impl SongLibraryScan {
    pub fn poll(&mut self) -> Option<SongLibrary> {
        match self.task.poll() {
            BackgroundPoll::Ready(library) => {
                self.scanning = false;
                Some(library)
            }
            BackgroundPoll::Disconnected => {
                self.scanning = false;
                None
            }
            BackgroundPoll::Pending => None,
        }
    }
}

pub fn start_library_scan(chart_root: &str, cache_path: &Path) -> (SongLibrary, SongLibraryScan) {
    let initial = load_cached_library(cache_path, chart_root).unwrap_or_else(empty_library);
    let root = chart_root.to_string();
    let mut task = BackgroundTask::default();
    task.start(move || scan_song_library(&root));
    (
        initial,
        SongLibraryScan {
            scanning: true,
            task,
        },
    )
}

fn empty_library() -> SongLibrary {
    SongLibrary {
        entries: Vec::new(),
        selected_entry: 0,
        selected_chart: 0,
        search: String::new(),
        sort_mode: SongSortMode::Title,
    }
}
