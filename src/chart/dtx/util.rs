use anyhow::{Result, anyhow};

pub fn normalized_pairs(value: &str) -> Vec<u8> {
    value
        .bytes()
        .filter(|b| *b != b'_' && !b.is_ascii_whitespace())
        .map(|b| b.to_ascii_uppercase())
        .collect()
}

pub fn parse_float(value: &str) -> Result<f32> {
    Ok(value.replace(',', ".").parse::<f32>()?)
}

pub fn base36_pair(pair: &[u8]) -> Result<u32> {
    Ok(base36_digit(pair[0])? * 36 + base36_digit(pair[1])?)
}

pub fn base36_str(value: &str) -> Result<u32> {
    let bytes = value.as_bytes();
    if bytes.len() != 2 {
        return Err(anyhow!("base36 id must be two chars"));
    }
    base36_pair(bytes)
}

fn base36_digit(byte: u8) -> Result<u32> {
    match byte.to_ascii_uppercase() {
        b'0'..=b'9' => Ok((byte - b'0') as u32),
        b'A'..=b'Z' => Ok((byte.to_ascii_uppercase() - b'A' + 10) as u32),
        _ => Err(anyhow!("invalid base36 digit")),
    }
}

pub fn command_index(command: &str, prefix: &str, suffix: &str) -> Option<usize> {
    let upper = command.to_ascii_uppercase();
    if !upper.starts_with(prefix) || !upper.ends_with(suffix) {
        return None;
    }
    upper[prefix.len()..upper.len() - suffix.len()]
        .parse::<usize>()
        .ok()
}
