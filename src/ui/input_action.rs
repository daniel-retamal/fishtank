use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

pub const KEY_NAMES: &[(&str, KeyCode)] = &[
    ("enter", KeyCode::Enter),
    ("esc", KeyCode::Esc),
    ("tab", KeyCode::Tab),
    ("backtab", KeyCode::BackTab),
    ("backspace", KeyCode::Backspace),
    ("delete", KeyCode::Delete),
    ("insert", KeyCode::Insert),
    ("up", KeyCode::Up),
    ("down", KeyCode::Down),
    ("left", KeyCode::Left),
    ("right", KeyCode::Right),
    ("home", KeyCode::Home),
    ("end", KeyCode::End),
    ("pageup", KeyCode::PageUp),
    ("pagedown", KeyCode::PageDown),
    ("space", KeyCode::Char(' ')),
];

pub enum InputAction {
    Quit,
    Cancel,
    Confirm,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Char(char),
    Backspace,
    Delete,
    Tab,
}

pub fn classify(event: &Event) -> Option<InputAction> {
    let Event::Key(key) = event else { return None };
    if key.kind != KeyEventKind::Press {
        return None;
    }
    Some(match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => InputAction::Quit,
        KeyCode::Esc => InputAction::Cancel,
        KeyCode::Enter => InputAction::Confirm,
        KeyCode::Up => InputAction::Up,
        KeyCode::Down => InputAction::Down,
        KeyCode::Left => InputAction::Left,
        KeyCode::Right => InputAction::Right,
        KeyCode::PageUp => InputAction::PageUp,
        KeyCode::PageDown => InputAction::PageDown,
        KeyCode::Home => InputAction::Home,
        KeyCode::End => InputAction::End,
        KeyCode::Char(c) => InputAction::Char(c),
        KeyCode::Backspace => InputAction::Backspace,
        KeyCode::Delete => InputAction::Delete,
        KeyCode::Tab => InputAction::Tab,
        _ => return None,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hold {
    pub code: KeyCode,
    pub down: bool,
    pub quits: bool,
}

pub fn hold(event: &Event) -> Option<Hold> {
    let Event::Key(key) = event else { return None };
    let down = key.kind != KeyEventKind::Release;
    let quits =
        down && key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
    Some(Hold {
        code: key.code,
        down,
        quits,
    })
}

pub fn key_code(name: &str) -> Option<KeyCode> {
    let lower = name.trim().to_lowercase();
    if let Some(&(_, code)) = KEY_NAMES.iter().find(|(named, _)| *named == lower) {
        return Some(code);
    }
    let mut chars = lower.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) => Some(KeyCode::Char(ch)),
        _ => None,
    }
}

pub fn key_name(code: KeyCode) -> Option<String> {
    if let Some(&(name, _)) = KEY_NAMES.iter().find(|(_, named)| *named == code) {
        return Some(name.to_string());
    }
    let KeyCode::Char(ch) = code else {
        return None;
    };
    Some(ch.to_lowercase().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    fn event(code: KeyCode, kind: KeyEventKind) -> Event {
        let mut key = KeyEvent::new(code, KeyModifiers::empty());
        key.kind = kind;
        Event::Key(key)
    }

    #[test]
    fn every_named_key_spells_itself_both_ways() {
        for &(name, code) in KEY_NAMES {
            assert_eq!(key_code(name), Some(code), "{name}");
            assert_eq!(key_name(code).as_deref(), Some(name), "{name}");
        }
    }

    #[test]
    fn any_other_character_is_itself_lowercased_so_shift_never_strands_a_key() {
        assert_eq!(key_name(KeyCode::Char('W')).as_deref(), Some("w"));
        assert_eq!(key_name(KeyCode::Char('w')).as_deref(), Some("w"));
        assert_eq!(key_code(" W "), Some(KeyCode::Char('w')));
        assert_eq!(
            key_code("spacebar"),
            None,
            "a word that is no key names none"
        );
    }

    #[test]
    fn a_press_and_a_repeat_hold_a_key_down_and_a_release_lets_it_go() {
        for (kind, down) in [
            (KeyEventKind::Press, true),
            (KeyEventKind::Repeat, true),
            (KeyEventKind::Release, false),
        ] {
            let held = hold(&event(KeyCode::Up, kind)).expect("a key event");
            assert_eq!(held.down, down, "{kind:?}");
            assert!(!held.quits);
        }
        assert_eq!(hold(&Event::FocusLost), None);
    }

    #[test]
    fn only_a_pressed_control_c_quits() {
        let mut key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(hold(&Event::Key(key)).expect("a key").quits);
        key.kind = KeyEventKind::Release;
        assert!(!hold(&Event::Key(key)).expect("a key").quits);
    }

    #[test]
    fn classify_still_ignores_everything_but_a_press() {
        assert!(classify(&event(KeyCode::Char('x'), KeyEventKind::Release)).is_none());
        assert!(classify(&event(KeyCode::Char('x'), KeyEventKind::Repeat)).is_none());
        assert!(classify(&event(KeyCode::Char('x'), KeyEventKind::Press)).is_some());
    }
}
