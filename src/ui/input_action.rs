use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

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
