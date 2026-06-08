use bevy::prelude::*;

pub const LANE_BD: usize = 0;
pub const LANE_SD: usize = 1;
pub const LANE_FT: usize = 2;
pub const LANE_HH: usize = 3;
pub const LANE_LP: usize = 4;
pub const LANE_LT: usize = 5;
pub const LANE_HT: usize = 6;
pub const LANE_CY: usize = 7;
pub const LANE_RD: usize = 8;
pub const LANE_LC: usize = 9;

pub const DTX_CH_HH_CLOSE: u32 = 0x11;
pub const DTX_CH_HH_OPEN: u32 = 0x18;
pub const DTX_CH_LP: u32 = 0x1B;
pub const DTX_CH_SE_HH: u32 = 0x84;
pub const DTX_TICKS_PER_MEASURE: u32 = 384;
pub const POLYPHONIC_VOICES: usize = 4;
pub const HH_TRACKED_WAV_CAP: usize = 16;

/// Screen left-to-right order matching GM drum note positions on a piano keyboard.
pub const LANE_DISPLAY_ORDER: [usize; 10] = [0, 1, 2, 3, 5, 6, 7, 8, 4, 9];

#[derive(Clone, Copy)]
pub struct LaneSpec {
    pub label: &'static str,
    pub key: &'static str,
    pub key_code: KeyCode,
    pub gm_drum_key: &'static str,
    pub gm_melodic_key: &'static str,
    pub color: Color,
}

impl LaneSpec {
    pub const fn new(
        label: &'static str,
        key: &'static str,
        key_code: KeyCode,
        gm_drum_key: &'static str,
        gm_melodic_key: &'static str,
        color: Color,
    ) -> Self {
        Self {
            label,
            key,
            key_code,
            gm_drum_key,
            gm_melodic_key,
            color,
        }
    }
}

pub const LANES: [LaneSpec; 10] = [
    LaneSpec::new(
        "BD",
        "A",
        KeyCode::KeyA,
        "C1",
        "C4",
        Color::srgb(0.95, 0.25, 0.25),
    ),
    LaneSpec::new(
        "SD",
        "S",
        KeyCode::KeyS,
        "D1",
        "D4",
        Color::srgb(1.00, 0.95, 0.75),
    ),
    LaneSpec::new(
        "FT",
        "D",
        KeyCode::KeyD,
        "F1",
        "E4",
        Color::srgb(0.75, 0.45, 1.00),
    ),
    LaneSpec::new(
        "HH",
        "F",
        KeyCode::KeyF,
        "F#1",
        "F4",
        Color::srgb(0.85, 0.95, 0.30),
    ),
    LaneSpec::new(
        "LP",
        "L",
        KeyCode::KeyL,
        "G2",
        "G4",
        Color::srgb(0.55, 0.80, 1.00),
    ),
    LaneSpec::new(
        "LT",
        "G",
        KeyCode::KeyG,
        "A1",
        "A4",
        Color::srgb(1.00, 0.62, 0.25),
    ),
    LaneSpec::new(
        "HT",
        "H",
        KeyCode::KeyH,
        "C2",
        "B4",
        Color::srgb(1.00, 0.45, 0.30),
    ),
    LaneSpec::new(
        "CY",
        "J",
        KeyCode::KeyJ,
        "C#2",
        "C5",
        Color::srgb(0.30, 0.95, 0.55),
    ),
    LaneSpec::new(
        "RD",
        "K",
        KeyCode::KeyK,
        "D#2",
        "D5",
        Color::srgb(0.30, 0.70, 1.00),
    ),
    LaneSpec::new(
        "LC",
        ";",
        KeyCode::Semicolon,
        "A2",
        "E5",
        Color::srgb(0.95, 0.35, 0.90),
    ),
];

pub fn lane_display_slot(lane: usize) -> usize {
    LANE_DISPLAY_ORDER
        .iter()
        .position(|&l| l == lane)
        .unwrap_or(lane)
}

pub fn lane_to_dtx_channel(lane: usize) -> u32 {
    match lane {
        LANE_BD => 0x13,
        LANE_SD => 0x12,
        LANE_FT => 0x17,
        LANE_HH => DTX_CH_HH_CLOSE,
        LANE_LP => DTX_CH_LP,
        LANE_LT => 0x15,
        LANE_HT => 0x14,
        LANE_CY => 0x16,
        LANE_RD => 0x19,
        LANE_LC => 0x1A,
        _ => DTX_CH_HH_CLOSE,
    }
}

pub fn dtx_drum_channel_to_lane(channel: u32) -> Option<usize> {
    match channel {
        0x13 | 0x1C => Some(LANE_BD),
        0x12 => Some(LANE_SD),
        0x17 => Some(LANE_FT),
        0x11 | 0x18 => Some(LANE_HH),
        0x1B => Some(LANE_LP),
        0x15 => Some(LANE_LT),
        0x14 => Some(LANE_HT),
        0x16 => Some(LANE_CY),
        0x19 => Some(LANE_RD),
        0x1A => Some(LANE_LC),
        _ => None,
    }
}

/// NoChip channels (`0xB1`–`0xBE`): chart-defined empty-pad hit sounds.
pub fn dtx_nosound_channel_to_lane(channel: u32) -> Option<usize> {
    match channel {
        0xB1 | 0xB8 => Some(LANE_HH),
        0xB2 => Some(LANE_SD),
        0xB3 | 0xBE => Some(LANE_BD),
        0xB4 => Some(LANE_HT),
        0xB5 => Some(LANE_LT),
        0xB6 => Some(LANE_CY),
        0xB7 => Some(LANE_FT),
        0xB9 => Some(LANE_RD),
        0xBC => Some(LANE_LC),
        0xBD => Some(LANE_LP),
        _ => None,
    }
}

pub fn dtx_override_se_to_lane(channel: u32) -> Option<usize> {
    match channel {
        0x84 => Some(LANE_HH),
        0x85 => Some(LANE_CY),
        0x86 => Some(LANE_RD),
        0x87 => Some(LANE_LC),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadGroup {
    Hh,
    Tom,
    Cymbal,
    Bd,
}

pub fn lane_pad_group(lane: usize) -> Option<PadGroup> {
    match lane {
        LANE_HH | LANE_LC => Some(PadGroup::Hh),
        LANE_LT | LANE_FT => Some(PadGroup::Tom),
        LANE_CY | LANE_RD => Some(PadGroup::Cymbal),
        LANE_BD | LANE_LP => Some(PadGroup::Bd),
        _ => None,
    }
}

pub fn pad_group_lanes_for_search(
    group: PadGroup,
    hit_lane: usize,
    chart_has_lane: impl Fn(usize) -> bool,
) -> Vec<usize> {
    match group {
        PadGroup::Hh => {
            let mut lanes = Vec::new();
            if chart_has_lane(LANE_HH) || hit_lane == LANE_HH {
                lanes.push(LANE_HH);
            }
            if chart_has_lane(LANE_LC) || hit_lane == LANE_LC {
                lanes.push(LANE_LC);
            }
            if hit_lane == LANE_LC && !chart_has_lane(LANE_LC) && !lanes.contains(&LANE_HH) {
                lanes.push(LANE_HH);
            }
            if lanes.is_empty() {
                lanes.extend([LANE_HH, LANE_LC]);
            }
            lanes
        }
        PadGroup::Tom => vec![LANE_LT, LANE_FT],
        PadGroup::Cymbal => {
            let mut lanes = Vec::new();
            if chart_has_lane(LANE_CY) || hit_lane == LANE_CY {
                lanes.push(LANE_CY);
            }
            if chart_has_lane(LANE_RD) || hit_lane == LANE_RD {
                lanes.push(LANE_RD);
            }
            if hit_lane == LANE_RD && !chart_has_lane(LANE_RD) && !lanes.contains(&LANE_CY) {
                lanes.push(LANE_CY);
            }
            if lanes.is_empty() {
                lanes.extend([LANE_CY, LANE_RD]);
            }
            lanes
        }
        PadGroup::Bd => vec![LANE_BD, LANE_LP],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nosound_channels_map_to_lanes() {
        assert_eq!(dtx_nosound_channel_to_lane(0xB1), Some(LANE_HH));
        assert_eq!(dtx_nosound_channel_to_lane(0xB8), Some(LANE_HH));
        assert_eq!(dtx_nosound_channel_to_lane(0xB3), Some(LANE_BD));
        assert_eq!(dtx_nosound_channel_to_lane(0xBE), Some(LANE_BD));
        assert_eq!(dtx_nosound_channel_to_lane(0xBC), Some(LANE_LC));
        assert_eq!(dtx_nosound_channel_to_lane(0xBA), None);
    }

    #[test]
    fn dtx_se24_to_se27_route_to_drum_lanes() {
        assert_eq!(dtx_override_se_to_lane(0x84), Some(LANE_HH));
        assert_eq!(dtx_override_se_to_lane(0x85), Some(LANE_CY));
        assert_eq!(dtx_override_se_to_lane(0x86), Some(LANE_RD));
        assert_eq!(dtx_override_se_to_lane(0x87), Some(LANE_LC));
    }

    #[test]
    fn ordinary_se_channels_do_not_route_to_drum_lanes() {
        assert_eq!(dtx_override_se_to_lane(0x61), None);
        assert_eq!(dtx_override_se_to_lane(0x65), None);
        assert_eq!(dtx_override_se_to_lane(0x90), None);
    }

    #[test]
    fn pad_group_lane_mapping() {
        assert_eq!(lane_pad_group(LANE_HH), Some(PadGroup::Hh));
        assert_eq!(lane_pad_group(LANE_LC), Some(PadGroup::Hh));
        assert_eq!(lane_pad_group(LANE_LT), Some(PadGroup::Tom));
        assert_eq!(lane_pad_group(LANE_FT), Some(PadGroup::Tom));
        assert_eq!(lane_pad_group(LANE_CY), Some(PadGroup::Cymbal));
        assert_eq!(lane_pad_group(LANE_RD), Some(PadGroup::Cymbal));
        assert_eq!(lane_pad_group(LANE_BD), Some(PadGroup::Bd));
        assert_eq!(lane_pad_group(LANE_LP), Some(PadGroup::Bd));
        assert_eq!(lane_pad_group(LANE_SD), None);
    }

    #[test]
    fn rd_without_ride_notes_falls_back_to_cymbal_lane() {
        let lanes = pad_group_lanes_for_search(PadGroup::Cymbal, LANE_RD, |_| false);
        assert_eq!(lanes, vec![LANE_RD, LANE_CY]);
    }

    #[test]
    fn lc_without_left_cymbal_notes_falls_back_to_hh_lane() {
        let lanes = pad_group_lanes_for_search(PadGroup::Hh, LANE_LC, |_| false);
        assert_eq!(lanes, vec![LANE_LC, LANE_HH]);
    }
}
