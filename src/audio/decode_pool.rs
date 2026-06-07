use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use bevy::log::info;
use kira::sound::static_sound::StaticSoundData;

use super::sound_bank::{BackgroundDecodeResult, DeferredWavEntry};

const DECODE_WORKERS: usize = 4;

pub fn spawn_bounded_decode(
    entries: Vec<DeferredWavEntry>,
) -> (mpsc::Receiver<BackgroundDecodeResult>, u32) {
    let pending = entries.len() as u32;
    let queue = Arc::new(Mutex::new(VecDeque::from(entries)));
    let (result_tx, result_rx) = mpsc::channel::<BackgroundDecodeResult>();

    let workers = DECODE_WORKERS.min(pending.max(1) as usize);
    info!("spawning {workers} background decode workers for {pending} deferred WAV files");

    for _ in 0..workers {
        let queue = Arc::clone(&queue);
        let result_tx = result_tx.clone();
        thread::spawn(move || decode_worker(queue, result_tx));
    }
    drop(result_tx);

    (result_rx, pending)
}

fn decode_worker(
    queue: Arc<Mutex<VecDeque<DeferredWavEntry>>>,
    result_tx: mpsc::Sender<BackgroundDecodeResult>,
) {
    loop {
        let entry = {
            let mut q = queue.lock().unwrap();
            q.pop_front()
        };
        let Some(entry) = entry else {
            break;
        };
        let result = BackgroundDecodeResult {
            id: entry.id,
            filename: entry.filename,
            sound: StaticSoundData::from_file(&entry.path),
            volume: entry.volume,
            pan: entry.pan,
            role: entry.role,
        };
        result_tx.send(result).ok();
    }
}
