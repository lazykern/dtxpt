pub fn is_dtx_se_channel(channel: u32) -> bool {
    matches!(channel, 0x61..=0x69 | 0x70..=0x79 | 0x80..=0x89 | 0x90..=0x92)
}

pub fn is_dtx_stick_se_channel(channel: u32) -> bool {
    (0x61..=0x65).contains(&channel)
}

pub fn dtx_wav_volume_command_id(command: &str) -> Option<u32> {
    let rest = command
        .strip_prefix("WAVVOL")
        .or_else(|| command.strip_prefix("wavvol"))
        .or_else(|| command.strip_prefix("VOLUME"))
        .or_else(|| command.strip_prefix("volume"))?;
    (rest.len() == 2)
        .then(|| super::util::base36_str(rest).ok())
        .flatten()
}

pub fn dtx_wav_pan_command_id(command: &str, value: &str) -> Option<u32> {
    let rest = if let Some(rest) = command.strip_prefix("WAVPAN") {
        Some(rest)
    } else if let Some(rest) = command.strip_prefix("wavpan") {
        Some(rest)
    } else if let Some(rest) = command.strip_prefix("PAN") {
        Some(rest)
    } else if let Some(rest) = command.strip_prefix("pan") {
        Some(rest)
    } else if let Some(rest) = command.strip_prefix("PANEL") {
        value.parse::<i32>().ok()?;
        Some(rest)
    } else if let Some(rest) = command.strip_prefix("panel") {
        value.parse::<i32>().ok()?;
        Some(rest)
    } else {
        None
    }?;
    (rest.len() == 2)
        .then(|| super::util::base36_str(rest).ok())
        .flatten()
}
