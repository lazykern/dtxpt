use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::chart::model::{
    BgaEvent, BgaImageDef, Chart, ChartNote, EmptyHitEvent, LongNote, NoteState, ScheduledAudio,
    ScheduledAudioKind, WavInfo, WavRole,
};
use crate::chart::timing::{ChartTiming, ChipPlayTimeComputeMode};
use crate::input::lanes::{
    DTX_TICKS_PER_MEASURE, dtx_bass_channel_to_lane, dtx_drum_channel_to_lane,
    dtx_guitar_channel_to_lane, dtx_nosound_channel_to_lane, is_bass_channel, is_guitar_channel,
};

use super::channels::{dtx_wav_pan_command_id, dtx_wav_volume_command_id, is_dtx_se_channel};
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
    GuitarNote {
        tick: u32,
        lane: usize,
        channel: u32,
        wav: Option<u32>,
    },
    BassNote {
        tick: u32,
        lane: usize,
        channel: u32,
        wav: Option<u32>,
    },
    GuitarLongNoteStart {
        tick: u32,
        lane: usize,
        channel: u32,
        wav: Option<u32>,
    },
    BassLongNoteStart {
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
    Bga {
        tick: u32,
        layer: u8,
        bmp_id: u32,
    },
    EmptyHit {
        tick: u32,
        lane: usize,
        channel: u32,
        wav: Option<u32>,
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

fn dtx_bga_channel_to_layer(channel: u32) -> Option<u8> {
    match channel {
        0x04 => Some(1),
        0x07 => Some(2),
        0x55 => Some(3),
        0x56 => Some(4),
        0x57 => Some(5),
        0x58 => Some(6),
        0x59 => Some(7),
        0x60 => Some(8),
        0xC1 => Some(3),
        0xC2 => Some(4),
        0xC3 => Some(5),
        0xC4 => Some(6),
        0xC5 => Some(7),
        0xC6 => Some(8),
        _ => None,
    }
}

pub fn parse_dtx_chart(text: &str, source: &str, chart_dir: &str) -> Result<(Chart, ChartTiming)> {
    parse_dtx_chart_with_compute_mode(text, source, chart_dir, ChipPlayTimeComputeMode::default())
}

pub fn parse_dtx_chart_with_compute_mode(
    text: &str,
    source: &str,
    chart_dir: &str,
    chip_play_time_compute_mode: ChipPlayTimeComputeMode,
) -> Result<(Chart, ChartTiming)> {
    let mut title = std::path::Path::new(source)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| source.to_string());
    let mut base_bpm = 120.0_f32;
    let mut level_raw: Option<(f32, bool)> = None;
    let mut level_dec = 0_i32;
    let mut bpm_defs: HashMap<u32, f32> = HashMap::new();
    let mut wav_files = Vec::new();
    let mut bga_images = Vec::new();
    let mut background_image = None;
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
        if command.eq_ignore_ascii_case("DLEVEL") {
            if let Ok(parsed) = parse_float(value) {
                level_raw = Some((parsed, value.contains('.')));
            }
            continue;
        }
        if command.eq_ignore_ascii_case("DLVDEC") {
            if let Ok(parsed) = value.parse::<i32>() {
                level_dec = parsed;
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
        if command.len() == 5 && command[..3].eq_ignore_ascii_case("BMP") {
            let rest = &command[3..];
            if let Ok(id) = base36_str(rest) {
                bga_images.push(BgaImageDef {
                    id,
                    filename: value.to_string(),
                });
            }
            continue;
        }
        if command.eq_ignore_ascii_case("BACKGROUND") || command.eq_ignore_ascii_case("STAGEFILE") {
            background_image = Some(value.to_string());
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
                } else if let Some(layer) = dtx_bga_channel_to_layer(channel) {
                    if let Ok(bmp_id) = base36_pair(pair) {
                        events.push(DtxEvent::Bga {
                            tick,
                            layer,
                            bmp_id,
                        });
                    }
                } else if is_dtx_se_channel(channel) {
                    if let Ok(wav) = base36_pair(pair) {
                        events.push(DtxEvent::AutoSe { tick, channel, wav });
                    }
                } else if let Some(lane) = dtx_nosound_channel_to_lane(channel) {
                    let wav = base36_pair(pair).ok();
                    events.push(DtxEvent::EmptyHit {
                        tick,
                        lane,
                        channel,
                        wav,
                    });
                } else if let Some(lane) = dtx_drum_channel_to_lane(channel) {
                    let wav = base36_pair(pair).ok();
                    events.push(DtxEvent::Note {
                        tick,
                        lane,
                        channel,
                        wav,
                    });
                } else if is_guitar_channel(channel) {
                    let lane = dtx_guitar_channel_to_lane(channel).unwrap_or(0);
                    let wav = base36_pair(pair).ok();
                    // Guitar_LongNote (0x2C=44) marks a long-note start;
                    // the end comes from the next chip on the same lane
                    // with a different channel. We track starts here and
                    // pair them up during the final pass.
                    if channel == 0x2C {
                        events.push(DtxEvent::GuitarLongNoteStart {
                            tick,
                            lane,
                            channel,
                            wav,
                        });
                    } else {
                        events.push(DtxEvent::GuitarNote {
                            tick,
                            lane,
                            channel,
                            wav,
                        });
                    }
                } else if is_bass_channel(channel) {
                    let lane = dtx_bass_channel_to_lane(channel).unwrap_or(0);
                    let wav = base36_pair(pair).ok();
                    // Bass_LongNote (0xAD=173) marks a long-note start.
                    if channel == 0xAD {
                        events.push(DtxEvent::BassLongNoteStart {
                            tick,
                            lane,
                            channel,
                            wav,
                        });
                    } else {
                        events.push(DtxEvent::BassNote {
                            tick,
                            lane,
                            channel,
                            wav,
                        });
                    }
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

    let skill_level = level_raw
        .map(|(level, decimal)| normalize_skill_level(level, decimal, level_dec))
        .unwrap_or(0.0);

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
    let timing = ChartTiming::with_compute_mode(
        base_bpm,
        tempo_events,
        end_tick,
        chip_play_time_compute_mode,
    );

    let mut wav_roles: HashMap<u32, WavRole> = HashMap::new();
    if let Some(wav) = bgm_wav {
        wav_roles.insert(wav, WavRole::Bgm);
    }
    for event in &events {
        match *event {
            DtxEvent::Note { wav: Some(wav), .. } => {
                merge_wav_role(&mut wav_roles, wav, WavRole::Drum);
            }
            DtxEvent::GuitarNote { wav: Some(wav), .. } => {
                merge_wav_role(&mut wav_roles, wav, WavRole::Guitar);
            }
            DtxEvent::BassNote { wav: Some(wav), .. } => {
                merge_wav_role(&mut wav_roles, wav, WavRole::Bass);
            }
            DtxEvent::GuitarLongNoteStart { wav: Some(wav), .. } => {
                merge_wav_role(&mut wav_roles, wav, WavRole::Guitar);
            }
            DtxEvent::BassLongNoteStart { wav: Some(wav), .. } => {
                merge_wav_role(&mut wav_roles, wav, WavRole::Bass);
            }
            DtxEvent::EmptyHit { wav: Some(wav), .. } => {
                merge_wav_role(&mut wav_roles, wav, WavRole::Drum);
            }
            DtxEvent::Bgm { wav, .. } => {
                merge_wav_role(&mut wav_roles, bgm_wav.unwrap_or(wav), WavRole::Bgm);
            }
            DtxEvent::AutoSe { wav, .. } => {
                merge_wav_role(&mut wav_roles, wav, WavRole::Se);
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
                autoplayed: false,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut guitar_notes = events
        .iter()
        .filter_map(|event| match *event {
            DtxEvent::GuitarNote {
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
                autoplayed: false,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut bass_notes = events
        .iter()
        .filter_map(|event| match *event {
            DtxEvent::BassNote {
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
                autoplayed: false,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();

    notes.sort_by(|a, b| a.time.total_cmp(&b.time).then(a.lane.cmp(&b.lane)));
    guitar_notes.sort_by(|a, b| a.time.total_cmp(&b.time).then(a.lane.cmp(&b.lane)));
    bass_notes.sort_by(|a, b| a.time.total_cmp(&b.time).then(a.lane.cmp(&b.lane)));

    // Pair GuitarLongNoteStart events with the next GuitarNote on the
    // same lane (the 'end' of the long note). This is a simplified
    // approximation of BocuD's long-note semantics; the exact end
    // channel/timing comes from the chart's tempo + the WAV's
    // duration, computed in the gameplay plugin.
    let mut guitar_long_notes: Vec<LongNote> = Vec::new();
    let mut bass_long_notes: Vec<LongNote> = Vec::new();
    {
        let mut guitar_starts: Vec<(usize, f32, u32, Option<u32>)> = Vec::new();
        for event in &events {
            if let DtxEvent::GuitarLongNoteStart {
                tick,
                lane,
                channel,
                wav,
            } = event
            {
                guitar_starts.push((*lane, timing.time_at_tick(*tick), *channel, *wav));
            }
        }
        guitar_starts.sort_by(|a, b| a.1.total_cmp(&b.1));
        for (lane, start_time, channel, wav) in guitar_starts {
            let end_time = guitar_notes
                .iter()
                .filter(|n| n.lane == lane && n.time > start_time)
                .map(|n| n.time)
                .next()
                .unwrap_or(start_time + 0.5); // default 500ms sustain
            guitar_long_notes.push(LongNote {
                start_time,
                end_time,
                lane,
                channel,
                wav_id: wav,
                state: NoteState::Pending,
            });
        }
        let mut bass_starts: Vec<(usize, f32, u32, Option<u32>)> = Vec::new();
        for event in &events {
            if let DtxEvent::BassLongNoteStart {
                tick,
                lane,
                channel,
                wav,
            } = event
            {
                bass_starts.push((*lane, timing.time_at_tick(*tick), *channel, *wav));
            }
        }
        bass_starts.sort_by(|a, b| a.1.total_cmp(&b.1));
        for (lane, start_time, channel, wav) in bass_starts {
            let end_time = bass_notes
                .iter()
                .filter(|n| n.lane == lane && n.time > start_time)
                .map(|n| n.time)
                .next()
                .unwrap_or(start_time + 0.5);
            bass_long_notes.push(LongNote {
                start_time,
                end_time,
                lane,
                channel,
                wav_id: wav,
                state: NoteState::Pending,
            });
        }
    }
    // The check allows guitar-only or bass-only charts (no drum chips)
    // and charts whose only chips are long-note starts.
    if notes.is_empty()
        && guitar_notes.is_empty()
        && bass_notes.is_empty()
        && events.iter().all(|event| {
            !matches!(
                event,
                DtxEvent::GuitarLongNoteStart { .. } | DtxEvent::BassLongNoteStart { .. }
            )
        })
    {
        return Err(anyhow!("no playable notes found (drum/guitar/bass)"));
    }

    let mut empty_hit_events = events
        .iter()
        .filter_map(|event| match *event {
            DtxEvent::EmptyHit {
                tick,
                lane,
                channel,
                wav,
            } => Some(EmptyHitEvent {
                time: timing.time_at_tick(tick),
                lane,
                channel,
                wav_id: wav,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    empty_hit_events.sort_by(|a, b| a.time.total_cmp(&b.time));

    let mut scheduled_audio = events
        .iter()
        .filter_map(|event| match *event {
            DtxEvent::Bgm { tick, wav } => Some(ScheduledAudio {
                time: timing.time_at_tick(tick),
                wav_id: bgm_wav.unwrap_or(wav),
                kind: ScheduledAudioKind::Bgm,
                fired: false,
            }),
            DtxEvent::AutoSe { tick, channel, wav } => Some(ScheduledAudio {
                time: timing.time_at_tick(tick),
                wav_id: wav,
                kind: ScheduledAudioKind::AutoSe { channel },
                fired: false,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    scheduled_audio.sort_by(|a, b| a.time.total_cmp(&b.time));

    let mut bga_events = events
        .iter()
        .filter_map(|event| match *event {
            DtxEvent::Bga {
                tick,
                layer,
                bmp_id,
            } => Some(BgaEvent {
                time: timing.time_at_tick(tick),
                layer,
                bmp_id,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    bga_events.sort_by(|a, b| a.time.total_cmp(&b.time).then(a.layer.cmp(&b.layer)));

    let metronome_beats = build_metronome_beats(&timing, end_tick);

    Ok((
        Chart {
            title,
            source: source.into(),
            bpm: base_bpm,
            skill_level,
            notes,
            guitar_notes,
            bass_notes,
            guitar_long_notes,
            bass_long_notes,
            empty_hit_events,
            metronome_beats,
            scheduled_audio,
            wav_info: wav_files,
            bga_images,
            bga_events,
            background_image,
            chart_dir: chart_dir.into(),
        },
        timing,
    ))
}

fn normalize_skill_level(level: f32, decimal: bool, level_dec: i32) -> f32 {
    if decimal {
        level
    } else if level >= 100.0 {
        level / 100.0
    } else {
        level / 10.0 + level_dec as f32 / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drum_backing_stem_auto_se_is_scheduled() {
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
        assert_eq!(chart.scheduled_audio.len(), 2);
        assert_eq!(chart.scheduled_audio[0].wav_id, 2);
        assert!(matches!(
            chart.scheduled_audio[0].kind,
            ScheduledAudioKind::AutoSe { channel: 0x61 }
        ));
        assert_eq!(chart.scheduled_audio[1].wav_id, 3);
        assert!(matches!(
            chart.scheduled_audio[1].kind,
            ScheduledAudioKind::AutoSe { channel: 0x65 }
        ));
        assert_eq!(
            chart.wav_info.iter().find(|wav| wav.id == 1).unwrap().role,
            WavRole::Drum
        );
        assert_eq!(
            chart.wav_info.iter().find(|wav| wav.id == 2).unwrap().role,
            WavRole::Se
        );
        assert_eq!(
            chart.wav_info.iter().find(|wav| wav.id == 3).unwrap().role,
            WavRole::Se
        );
    }

    #[test]
    fn nosound_channels_become_empty_hit_events() {
        let text = "\
#TITLE: nosound\n\
#BPM: 120\n\
#WAV01: kick.ogg\n\
#WAV02: snare.ogg\n\
#000B1: 01\n\
#001B2: 02\n\
#00011: 01\n";
        let (chart, _) = parse_dtx_chart(text, "test.dtx", ".").unwrap();

        assert_eq!(chart.empty_hit_events.len(), 2);
        assert_eq!(chart.empty_hit_events[0].lane, 3);
        assert_eq!(chart.empty_hit_events[0].channel, 0xB1);
        assert_eq!(chart.empty_hit_events[0].wav_id, Some(1));
        assert_eq!(chart.empty_hit_events[1].lane, 1);
        assert_eq!(chart.empty_hit_events[1].channel, 0xB2);
        assert_eq!(chart.empty_hit_events[1].wav_id, Some(2));
    }

    #[test]
    fn bmp_definitions_and_bga_channels_become_static_events() {
        let text = "\
#TITLE: bga\n\
#BPM: 120\n\
#WAV01: kick.ogg\n\
#BMP01: bg.png\n\
#BMP02: layer.png\n\
#BACKGROUND: back.jpg\n\
#00011: 01\n\
#00004: 01\n\
#00107: 02\n";
        let (chart, _) = parse_dtx_chart(text, "test.dtx", ".").unwrap();

        assert_eq!(chart.bga_images.len(), 2);
        assert_eq!(chart.bga_images[0].id, 1);
        assert_eq!(chart.bga_images[0].filename, "bg.png");
        assert_eq!(chart.background_image.as_deref(), Some("back.jpg"));
        assert_eq!(chart.bga_events.len(), 2);
        assert_eq!(chart.bga_events[0].layer, 1);
        assert_eq!(chart.bga_events[0].bmp_id, 1);
        assert_eq!(chart.bga_events[1].layer, 2);
        assert_eq!(chart.bga_events[1].bmp_id, 2);
    }

    #[test]
    fn dlevel_and_dlvdec_set_skill_level() {
        let text = "\
#TITLE: level\n\
#BPM: 120\n\
#DLEVEL: 85\n\
#DLVDEC: 7\n\
#WAV01: kick.ogg\n\
#00011: 01\n";
        let (chart, _) = parse_dtx_chart(text, "test.dtx", ".").unwrap();
        assert!((chart.skill_level - 8.57).abs() < 0.001);
    }

    #[test]
    fn decimal_dlevel_sets_skill_level_directly() {
        let text = "\
#TITLE: level\n\
#BPM: 120\n\
#DLEVEL: 8.5\n\
#DLVDEC: 7\n\
#WAV01: kick.ogg\n\
#00011: 01\n";
        let (chart, _) = parse_dtx_chart(text, "test.dtx", ".").unwrap();
        assert!((chart.skill_level - 8.5).abs() < 0.001);
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

    #[test]
    fn guitar_channels_populate_guitar_notes_slice() {
        // 0x22 = guitar G; 0x24 = guitar R; 0x28 = wailing.
        // Drum channels (0x11) must still populate `notes`.
        let text = "\
#TITLE: guitar chart\n\
#BPM: 120\n\
#WAV01: kick.ogg\n\
#WAV02: gtr.ogg\n\
#00011: 01\n\
#00022: 02\n\
#00024: 02\n\
#00028: 00\n";
        let (chart, _) = parse_dtx_chart(text, "guitar.dtx", ".").unwrap();

        // Drum note (0x11) stays in `notes`.
        assert_eq!(chart.notes.len(), 1);
        assert_eq!(chart.notes[0].channel, 0x11);
        // Guitar notes populate `guitar_notes` (2 notes: G + R; wailing
        // is captured as a long-note start, not a regular note).
        assert_eq!(chart.guitar_notes.len(), 2);
        let channels: Vec<u32> = chart.guitar_notes.iter().map(|n| n.channel).collect();
        assert!(channels.contains(&0x22));
        assert!(channels.contains(&0x24));
        assert!(!channels.contains(&0x28));
        // Bass slice is empty for a guitar-only chart.
        assert!(chart.bass_notes.is_empty());
        // WavRole for guitar note is WavRole::Guitar.
        let gtr_wav = chart.wav_info.iter().find(|wav| wav.id == 2).unwrap();
        assert_eq!(gtr_wav.role, WavRole::Guitar);
    }

    #[test]
    fn bass_channels_populate_bass_notes_slice() {
        // 0xA2 = bass G; 0xA4 = bass R; 0xA8 = bass wailing.
        let text = "\
#TITLE: bass chart\n\
#BPM: 120\n\
#WAV01: kick.ogg\n\
#WAV02: bass.ogg\n\
#00011: 01\n\
#000A2: 02\n\
#000A4: 02\n\
#000A8: 00\n";
        let (chart, _) = parse_dtx_chart(text, "bass.dtx", ".").unwrap();

        // Drum note (0x11) stays in `notes`.
        assert_eq!(chart.notes.len(), 1);
        // Bass notes populate `bass_notes` (2 notes: G + R; wailing is
        // captured as a long-note start, not a regular note).
        assert_eq!(chart.bass_notes.len(), 2);
        let channels: Vec<u32> = chart.bass_notes.iter().map(|n| n.channel).collect();
        assert!(channels.contains(&0xA2));
        assert!(channels.contains(&0xA4));
        assert!(!channels.contains(&0xA8));
        // Guitar slice is empty for a bass-only chart.
        assert!(chart.guitar_notes.is_empty());
        // WavRole for bass note is WavRole::Bass.
        let bass_wav = chart.wav_info.iter().find(|wav| wav.id == 2).unwrap();
        assert_eq!(bass_wav.role, WavRole::Bass);
    }

    #[test]
    fn guitar_long_note_start_is_captured_separately() {
        // 0x2C = Guitar_LongNote start. Parsed as a long-note start
        // and paired with the next guitar note on the same lane to
        // form a LongNote. The pair `00` means "no chip"; we use a
        // real WAV id here so the chip is captured.
        let text = "\
#TITLE: long note\n\
#BPM: 120\n\
#WAV01: kick.ogg\n\
#WAV02: gtr.ogg\n\
#00011: 01\n\
#0002C: 02\n\
#00122: 02\n";
        let (chart, _) = parse_dtx_chart(text, "long.dtx", ".").unwrap();
        // Drum note (0x11) populates `notes`.
        assert_eq!(chart.notes.len(), 1);
        // Regular guitar_notes vec has the end-of-long-note chip
        // (0x22 at measure 1, tick 384 = 0.5s at 120 BPM).
        assert_eq!(chart.guitar_notes.len(), 1);
        assert_eq!(chart.guitar_notes[0].channel, 0x22);
        // Long note is paired: start at t=0 (measure 0, tick 0),
        // end at t=0.5 (the next G chip on the same lane).
        assert_eq!(chart.guitar_long_notes.len(), 1);
        let long = &chart.guitar_long_notes[0];
        assert_eq!(long.start_time, 0.0);
        // Lane is the start chip's lane (WAIL for 0x2C, index 8 in
        // GUITAR_LANE_*). The end chip's lane is the visible lane
        // the player is sustaining; the gameplay plugin resolves
        // that via the start→end pairing at hit time.
        assert_eq!(long.channel, 0x2C);
        assert!(long.end_time > long.start_time);
        assert_eq!(long.state, NoteState::Pending);
    }

    #[test]
    fn long_note_without_end_uses_default_duration() {
        // No end chip → LongNote with default 500ms sustain.
        let text = "\
#TITLE: orphan long\n\
#BPM: 120\n\
#WAV01: kick.ogg\n\
#WAV02: gtr.ogg\n\
#00011: 01\n\
#0002C: 02\n";
        let (chart, _) = parse_dtx_chart(text, "orphan.dtx", ".").unwrap();
        assert_eq!(chart.guitar_long_notes.len(), 1);
        let long = &chart.guitar_long_notes[0];
        // Default end = start + 0.5s.
        assert!((long.end_time - long.start_time - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn bass_long_note_start_pairs_with_next_bass_note() {
        // 0xAD = Bass_LongNote start. Same pairing logic as guitar.
        let text = "\
#TITLE: bass long\n\
#BPM: 120\n\
#WAV01: kick.ogg\n\
#WAV02: bass.ogg\n\
#00011: 01\n\
#000AD: 02\n\
#001A2: 02\n";
        let (chart, _) = parse_dtx_chart(text, "bass_long.dtx", ".").unwrap();
        assert_eq!(chart.bass_long_notes.len(), 1);
        let long = &chart.bass_long_notes[0];
        assert_eq!(long.start_time, 0.0);
        assert!(long.end_time > long.start_time);
    }
}
