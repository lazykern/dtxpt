use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::chart::model::{
    Chart, ChartNote, NoteState, ScheduledAudio, ScheduledAudioKind, WavInfo, WavRole,
};
use crate::chart::timing::ChartTiming;
use crate::input::lanes::{dtx_drum_channel_to_lane, DTX_TICKS_PER_MEASURE};

use super::channels::{
    dtx_wav_pan_command_id, dtx_wav_volume_command_id, is_drum_backing_stem_wav, is_dtx_se_channel,
};
use super::metronome::build_metronome_beats;
use super::util::{base36_pair, base36_str, normalized_pairs, parse_float};

#[derive(Clone)]
enum DtxEvent {
    Note {
        tick: u32,
        lane: usize,
        channel: u32,
        wav: Option<u32>,
    },
    Bgm {
        tick: u32,
        wav: u32,
    },
    AutoSe {
        tick: u32,
        channel: u32,
        wav: u32,
    },
    Bpm {
        tick: u32,
        bpm: f32,
    },
    BarLength {
        tick: u32,
        ratio: f32,
    },
}

fn merge_wav_role(roles: &mut HashMap<u32, WavRole>, wav: u32, role: WavRole) {
    roles
        .entry(wav)
        .and_modify(|existing| *existing = existing.merge(role))
        .or_insert(role);
}

pub fn parse_dtx_chart(text: &str, source: &str, chart_dir: &str) -> Result<(Chart, ChartTiming)> {
    let mut title = std::path::Path::new(source)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| source.to_string());
    let mut base_bpm = 120.0_f32;
    let mut bpm_defs: HashMap<u32, f32> = HashMap::new();
    let mut wav_files = Vec::new();
    let mut wav_volumes: HashMap<u32, i32> = HashMap::new();
    let mut wav_pans: HashMap<u32, i32> = HashMap::new();
    let mut bgm_wav = None;
    let mut events = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.split(';').next().unwrap_or("").trim();
        if !line.starts_with('#') || line.len() < 2 {
            continue;
        }

        let body = &line[1..];
        let (command, value) = if let Some((command, value)) = body.split_once(':') {
            (command.trim(), value.trim())
        } else if let Some((command, value)) = body.split_once(' ') {
            (command.trim(), value.trim())
        } else {
            (body.trim(), "")
        };

        if command.eq_ignore_ascii_case("TITLE") {
            title = value.to_string();
            continue;
        }
        if command.eq_ignore_ascii_case("BPM") {
            if let Ok(parsed) = parse_float(value) {
                base_bpm = parsed;
            }
            continue;
        }
        if command.len() == 5 && command[..3].eq_ignore_ascii_case("WAV") {
            let rest = &command[3..];
            if let Ok(id) = base36_str(rest) {
                wav_files.push(WavInfo {
                    id,
                    filename: value.to_string(),
                    volume: *wav_volumes.get(&id).unwrap_or(&100),
                    pan: *wav_pans.get(&id).unwrap_or(&0),
                    role: WavRole::Drum,
                });
            }
            continue;
        }
        if let Some(id) = dtx_wav_volume_command_id(command) {
            if let Ok(volume) = value.parse::<i32>() {
                let volume = volume.clamp(0, 100);
                wav_volumes.insert(id, volume);
                for wav in wav_files.iter_mut().filter(|wav| wav.id == id) {
                    wav.volume = volume;
                }
            }
            continue;
        }
        if let Some(id) = dtx_wav_pan_command_id(command, value) {
            if let Ok(pan) = value.parse::<i32>() {
                let pan = pan.clamp(-100, 100);
                wav_pans.insert(id, pan);
                for wav in wav_files.iter_mut().filter(|wav| wav.id == id) {
                    wav.pan = pan;
                }
            }
            continue;
        }
        if command.eq_ignore_ascii_case("BGMWAV") {
            if let Ok(id) = base36_str(value.trim()) {
                bgm_wav = Some(id);
            }
            continue;
        }
        if command.len() == 5 && command[..3].chars().all(|c| c.is_ascii_digit()) {
            let measure = command[0..3].parse::<u32>()?;
            let measure_tick = measure * 384;
            let channel = u32::from_str_radix(&command[3..5], 16)?;
            if channel == 0x02 {
                events.push(DtxEvent::BarLength {
                    tick: measure_tick,
                    ratio: parse_float(value).unwrap_or(1.0),
                });
                continue;
            }
            let pairs = normalized_pairs(value);
            let count = pairs.len() / 2;
            if count == 0 {
                continue;
            }
            for (i, pair) in pairs.chunks(2).enumerate() {
                if pair.len() != 2 || pair == b"00" {
                    continue;
                }
                let tick = measure_tick + ((384 * i as u32) / count as u32);
                if channel == 0x01 {
                    if let Ok(wav) = base36_pair(pair) {
                        events.push(DtxEvent::Bgm { tick, wav });
                    }
                } else if channel == 0x03 {
                    if let Ok(bpm_delta) = u32::from_str_radix(std::str::from_utf8(pair)?, 16) {
                        events.push(DtxEvent::Bpm {
                            tick,
                            bpm: base_bpm + bpm_delta as f32,
                        });
                    }
                } else if channel == 0x08 {
                    if let Some(bpm) = bpm_defs.get(&base36_pair(pair)?) {
                        events.push(DtxEvent::Bpm { tick, bpm: *bpm });
                    }
                } else if is_dtx_se_channel(channel) {
                    if let Ok(wav) = base36_pair(pair) {
                        events.push(DtxEvent::AutoSe { tick, channel, wav });
                    }
                } else if let Some(lane) = dtx_drum_channel_to_lane(channel) {
                    let wav = base36_pair(pair).ok();
                    events.push(DtxEvent::Note {
                        tick,
                        lane,
                        channel,
                        wav,
                    });
                }
            }
            continue;
        }
        if command.len() == 5 && command[..3].eq_ignore_ascii_case("BPM") {
            let id = base36_str(&command[3..5])?;
            if let Ok(bpm) = parse_float(value) {
                bpm_defs.insert(id, bpm);
            }
        }
    }

    let tempo_events = events
        .iter()
        .filter_map(|event| match event {
            DtxEvent::Bpm { tick, bpm } => Some((*tick, *bpm)),
            DtxEvent::BarLength { tick, ratio } => Some((*tick, -*ratio)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let end_tick = events
        .iter()
        .filter_map(|event| match event {
            DtxEvent::Note { tick, .. } => Some(*tick),
            _ => None,
        })
        .max()
        .unwrap_or(DTX_TICKS_PER_MEASURE);
    let timing = ChartTiming::new(base_bpm, tempo_events, end_tick);

    let mut wav_roles: HashMap<u32, WavRole> = HashMap::new();
    if let Some(wav) = bgm_wav {
        wav_roles.insert(wav, WavRole::Bgm);
    }
    for event in &events {
        match *event {
            DtxEvent::Note { wav: Some(wav), .. } => {
                merge_wav_role(&mut wav_roles, wav, WavRole::Drum);
            }
            DtxEvent::Bgm { wav, .. } => {
                merge_wav_role(&mut wav_roles, bgm_wav.unwrap_or(wav), WavRole::Bgm);
            }
            DtxEvent::AutoSe { wav, .. } => {
                if !is_drum_backing_stem_wav(wav, &wav_files) {
                    merge_wav_role(&mut wav_roles, wav, WavRole::Se);
                }
            }
            _ => {}
        }
    }
    for wav in &mut wav_files {
        if let Some(role) = wav_roles.get(&wav.id) {
            wav.role = *role;
        }
    }

    let mut notes = events
        .iter()
        .filter_map(|event| match *event {
            DtxEvent::Note {
                tick,
                lane,
                channel,
                wav,
            } => Some(ChartNote {
                time: timing.time_at_tick(tick),
                lane,
                channel,
                wav_id: wav,
                state: NoteState::Pending,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();

    notes.sort_by(|a, b| a.time.total_cmp(&b.time).then(a.lane.cmp(&b.lane)));
    if notes.is_empty() {
        return Err(anyhow!("no playable drum notes found"));
    }

    let mut scheduled_audio = events
        .iter()
        .filter_map(|event| match *event {
            DtxEvent::Bgm { tick, wav } => Some(ScheduledAudio {
                time: timing.time_at_tick(tick),
                wav_id: bgm_wav.unwrap_or(wav),
                kind: ScheduledAudioKind::Bgm,
                fired: false,
            }),
            DtxEvent::AutoSe { tick, channel, wav } => {
                if is_drum_backing_stem_wav(wav, &wav_files) {
                    None
                } else {
                    Some(ScheduledAudio {
                        time: timing.time_at_tick(tick),
                        wav_id: wav,
                        kind: ScheduledAudioKind::AutoSe { channel },
                        fired: false,
                    })
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    scheduled_audio.sort_by(|a, b| a.time.total_cmp(&b.time));

    let metronome_beats = build_metronome_beats(&timing, end_tick);

    Ok((
        Chart {
            title,
            source: source.into(),
            bpm: base_bpm,
            notes,
            metronome_beats,
            scheduled_audio,
            wav_info: wav_files,
            chart_dir: chart_dir.into(),
        },
        timing,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drum_backing_stem_auto_se_is_skipped() {
        let text = "\
#TITLE: stem skip\n\
#BPM: 120\n\
#WAV01: kick.ogg\n\
#WAV02: drums.ogg\n\
#WAV03: cue.ogg\n\
#00011: 01\n\
#00061: 02\n\
#00065: 03\n";
        let (chart, _) = parse_dtx_chart(text, "test.dtx", ".").unwrap();

        assert_eq!(chart.notes.len(), 1);
        assert_eq!(chart.scheduled_audio.len(), 1);
        assert_eq!(chart.scheduled_audio[0].wav_id, 3);
        assert!(matches!(
            chart.scheduled_audio[0].kind,
            ScheduledAudioKind::AutoSe { channel: 0x65 }
        ));
        assert_eq!(
            chart.wav_info.iter().find(|wav| wav.id == 1).unwrap().role,
            WavRole::Drum
        );
        assert_eq!(
            chart.wav_info.iter().find(|wav| wav.id == 3).unwrap().role,
            WavRole::Se
        );
    }

    #[test]
    fn bgm_wav_role_uses_bgmwav_override() {
        let text = "\
#TITLE: bgm role\n\
#BPM: 120\n\
#WAV01: marker.ogg\n\
#WAV02: bgm.ogg\n\
#WAV03: kick.ogg\n\
#BGMWAV: 02\n\
#00001: 01\n\
#00011: 03\n";
        let (chart, _) = parse_dtx_chart(text, "test.dtx", ".").unwrap();

        assert_eq!(
            chart.wav_info.iter().find(|wav| wav.id == 2).unwrap().role,
            WavRole::Bgm
        );
        assert_eq!(
            chart.wav_info.iter().find(|wav| wav.id == 3).unwrap().role,
            WavRole::Drum
        );
    }
}
