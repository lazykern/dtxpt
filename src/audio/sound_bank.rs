use bevy::{asset::Handle, prelude::*};
use bevy_kira_audio::AudioSource;

use kira::sound::static_sound::StaticSoundData;

use dtxpt::chart::{Chart, WavRole};

#[derive(Resource)]
pub(crate) struct SoundBank {
    pub(crate) wavs: std::collections::HashMap<u32, LoadedWav>,
}

pub(crate) struct LoadedWav {
    pub(crate) handle: Handle<AudioSource>,
    pub volume: i32,
    pub pan: i32,
    pub role: WavRole,
}

pub(crate) struct DeferredWavEntry {
    pub id: u32,
    pub filename: String,
    pub path: std::path::PathBuf,
    pub volume: i32,
    pub pan: i32,
    pub role: WavRole,
}

pub(crate) struct BackgroundDecodeResult {
    pub(crate) id: u32,
    pub(crate) filename: String,
    pub(crate) sound: std::result::Result<StaticSoundData, kira::sound::FromFileError>,
    pub volume: i32,
    pub pan: i32,
    pub role: WavRole,
}

#[derive(Resource)]
pub(crate) struct BackgroundDecodeReceiver {
    pub(crate) rx: std::sync::Mutex<std::sync::mpsc::Receiver<BackgroundDecodeResult>>,
    pub(crate) pending: u32,
}

pub(crate) fn build_sound_bank_for_ids(
    chart: &Chart,
    asset_server: &AssetServer,
    ids: &std::collections::HashSet<u32>,
    label: &str,
) -> SoundBank {
    let mut wavs = std::collections::HashMap::new();
    if ids.is_empty() {
        info!("no {} WAV ids to load; no sounds will play", label);
        return SoundBank { wavs };
    }

    let file_index = build_chart_dir_file_index(&chart.chart_dir);
    let matching = chart
        .wav_info
        .iter()
        .filter(|wav| ids.contains(&wav.id))
        .count();
    info!(
        "loading {} {} WAV ids ({} defs total) from {}",
        matching,
        label,
        chart.wav_info.len(),
        chart.chart_dir
    );

    let mut handles_by_path =
        std::collections::HashMap::<std::path::PathBuf, Handle<AudioSource>>::new();
    let mut warned_missing = std::collections::HashSet::<std::path::PathBuf>::new();
    let mut warned_failed = std::collections::HashSet::<std::path::PathBuf>::new();
    let mut reused_handles = 0_usize;

    for wav in chart.wav_info.iter().filter(|wav| ids.contains(&wav.id)) {
        let path = resolve_chart_asset_path(&chart.chart_dir, &wav.filename, &file_index);
        if !path.exists() {
            if warned_missing.insert(path.clone()) {
                warn!(
                    "failed to load WAV {}: No such file or directory (os error 2)",
                    path.display()
                );
            }
            continue;
        }

        let handle = if let Some(existing) = handles_by_path.get(&path) {
            reused_handles += 1;
            existing.clone()
        } else {
            match StaticSoundData::from_file(&path) {
                Ok(sound) => {
                    let handle: Handle<AudioSource> = asset_server.add(AudioSource { sound });
                    handles_by_path.insert(path.clone(), handle.clone());
                    handle
                }
                Err(err) => {
                    if warned_failed.insert(path.clone()) {
                        warn!("failed to load WAV {}: {}", path.display(), err);
                    }
                    continue;
                }
            }
        };

        wavs.insert(
            wav.id,
            LoadedWav {
                handle,
                volume: wav.volume,
                pan: wav.pan,
                role: wav.role,
            },
        );
    }

    info!(
        "loaded {} {} WAV ids from {} unique files ({} handle reuses)",
        wavs.len(),
        label,
        handles_by_path.len(),
        reused_handles,
    );

    SoundBank { wavs }
}

pub(crate) fn collect_immediate_wav_ids(chart: &Chart) -> std::collections::HashSet<u32> {
    chart
        .notes
        .iter()
        .filter_map(|note| note.wav_id)
        .chain(
            chart
                .empty_hit_events
                .iter()
                .filter_map(|event| event.wav_id),
        )
        .collect()
}

pub(crate) fn collect_deferred_wav_ids(chart: &Chart) -> std::collections::HashSet<u32> {
    let note_ids: std::collections::HashSet<u32> =
        chart.notes.iter().filter_map(|note| note.wav_id).collect();
    let scheduled_ids: std::collections::HashSet<u32> = chart
        .scheduled_audio
        .iter()
        .map(|event| event.wav_id)
        .collect();
    scheduled_ids.difference(&note_ids).copied().collect()
}

pub(crate) fn build_deferred_entries(
    chart: &Chart,
    deferred_ids: &std::collections::HashSet<u32>,
) -> Vec<DeferredWavEntry> {
    let file_index = build_chart_dir_file_index(&chart.chart_dir);
    let mut entries = Vec::new();
    for wav in chart
        .wav_info
        .iter()
        .filter(|wav| deferred_ids.contains(&wav.id))
    {
        let path = resolve_chart_asset_path(&chart.chart_dir, &wav.filename, &file_index);
        entries.push(DeferredWavEntry {
            id: wav.id,
            filename: wav.filename.clone(),
            path,
            volume: wav.volume,
            pan: wav.pan,
            role: wav.role,
        });
    }
    entries
}

pub(crate) fn merge_decoded_audio(
    mut commands: Commands,
    mut sound_bank: ResMut<SoundBank>,
    receiver: Option<ResMut<BackgroundDecodeReceiver>>,
    mut playback_diag: ResMut<crate::gameplay::PlaybackDiagnostics>,
    run: Res<crate::gameplay::RunState>,
    asset_server: Res<AssetServer>,
) {
    let Some(mut receiver) = receiver else {
        return;
    };

    let results: Vec<BackgroundDecodeResult> = {
        let rx = receiver.rx.lock().unwrap();
        rx.try_iter().collect()
    };

    if results.is_empty() {
        return;
    }

    let mut merged = 0u32;
    for result in results {
        match result.sound {
            Ok(sound) => {
                let handle: Handle<AudioSource> = asset_server.add(AudioSource { sound });
                sound_bank.wavs.insert(
                    result.id,
                    LoadedWav {
                        handle,
                        volume: result.volume,
                        pan: result.pan,
                        role: result.role,
                    },
                );
            }
            Err(e) => warn!("bg decode failed {}: {}", result.filename, e),
        }
        merged += 1;
    }

    receiver.pending = receiver.pending.saturating_sub(merged);
    let pending = receiver.pending;
    if crate::gameplay::diagnostics::diag_active(&run) {
        playback_diag.bg_decode_merges += 1;
        debug!("merged {merged} background-decoded WAV files ({pending} pending)");
    }
    info!(
        "merged {} background-decoded WAV files ({} pending)",
        merged, pending
    );

    if pending == 0 {
        commands.remove_resource::<BackgroundDecodeReceiver>();
        info!("all background audio decodes complete");
    }
}

fn build_chart_dir_file_index(
    chart_dir: &str,
) -> std::collections::HashMap<String, std::path::PathBuf> {
    let mut index = std::collections::HashMap::new();
    if let Ok(entries) = std::fs::read_dir(chart_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                index
                    .entry(name.to_string_lossy().to_lowercase())
                    .or_insert(path);
            }
        }
    }
    index
}

fn resolve_chart_asset_path(
    chart_dir: &str,
    fname: &str,
    file_index: &std::collections::HashMap<String, std::path::PathBuf>,
) -> std::path::PathBuf {
    let direct = std::path::Path::new(chart_dir).join(fname);
    if direct.exists() {
        return direct;
    }

    file_index
        .get(&fname.to_lowercase())
        .cloned()
        .unwrap_or(direct)
}
