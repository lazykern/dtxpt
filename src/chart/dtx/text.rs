use std::path::Path;

use anyhow::Result;
use encoding_rs::SHIFT_JIS;

pub fn read_text(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(decode_bytes(&bytes))
}

pub fn decode_bytes(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(utf8) => utf8.to_string(),
        Err(_) => SHIFT_JIS.decode(bytes).0.into_owned(),
    }
}

pub fn parse_directive(raw: &str) -> Option<(String, &str)> {
    let line = raw.split(';').next().unwrap_or("").trim();
    if !line.starts_with('#') {
        return None;
    }
    let body = &line[1..];
    let (command, value) = if let Some((command, value)) = body.split_once(':') {
        (command.trim(), value.trim())
    } else if let Some((command, value)) = body.split_once(' ') {
        (command.trim(), value.trim())
    } else {
        return None;
    };
    Some((command.to_string(), value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_directive_with_colon_or_space() {
        assert_eq!(parse_directive("#TITLE: Song").unwrap().1, "Song");
        assert_eq!(parse_directive("#ARTIST Band").unwrap().1, "Band");
        assert!(parse_directive("; nope").is_none());
    }
}
