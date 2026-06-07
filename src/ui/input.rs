use std::time::Duration;

use bevy::prelude::*;

const KEY_REPEAT_DELAY: Duration = Duration::from_millis(260);
const KEY_REPEAT_INTERVAL: Duration = Duration::from_millis(70);

#[derive(Default)]
pub struct UiKeyRepeat {
    active_key: Option<KeyCode>,
    elapsed: Duration,
    repeating: bool,
}

impl UiKeyRepeat {
    pub fn update(
        &mut self,
        keyboard: &ButtonInput<KeyCode>,
        time: &Time,
        keys: &[KeyCode],
    ) -> Option<KeyCode> {
        for &key in keys {
            if keyboard.just_pressed(key) {
                self.active_key = Some(key);
                self.elapsed = Duration::ZERO;
                self.repeating = false;
                return Some(key);
            }
        }

        let key = self.active_key?;

        if keyboard.just_released(key) || !keyboard.pressed(key) {
            self.active_key = None;
            self.elapsed = Duration::ZERO;
            self.repeating = false;
            return None;
        }

        self.elapsed += time.delta();
        let threshold = if self.repeating {
            KEY_REPEAT_INTERVAL
        } else {
            KEY_REPEAT_DELAY
        };
        if self.elapsed < threshold {
            return None;
        }

        self.elapsed = self.elapsed.saturating_sub(threshold);
        self.repeating = true;
        Some(key)
    }
}

pub fn search_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}
