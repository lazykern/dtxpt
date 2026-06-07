use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static ENV_DIAG: OnceLock<bool> = OnceLock::new();
static MIDI_RESCAN_COUNTER: AtomicU32 = AtomicU32::new(0);

/// `DTXPT_DIAG=1` enables verbose playback diagnostics on stderr.
pub fn env_diag_enabled() -> bool {
    *ENV_DIAG.get_or_init(|| std::env::var("DTXPT_DIAG").is_ok())
}

pub fn record_midi_rescan() {
    MIDI_RESCAN_COUNTER.fetch_add(1, Ordering::Relaxed);
}

pub fn take_midi_rescan_count() -> u32 {
    MIDI_RESCAN_COUNTER.swap(0, Ordering::Relaxed)
}
