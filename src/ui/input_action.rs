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
    if key.kind == KeyEventKind::Release {
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

const UNLEARNED_FIRST_REPEAT_SECS: f32 = 0.5;
const UNLEARNED_REPEAT_SECS: f32 = 0.1;
const MAX_FIRST_REPEAT_SECS: f32 = 2.0;
const MAX_REPEAT_SECS: f32 = 0.5;
const REPEAT_SLACK: f32 = 1.25;
const STAMP_TICKS: f32 = 2.0;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub enum Releases {
    #[default]
    None,
    BeyondText,
    All,
}

impl Releases {
    fn heard_from(code: KeyCode) -> Self {
        if is_text_key(code) {
            Releases::All
        } else {
            Releases::BeyondText
        }
    }

    fn reported_for(self, code: KeyCode) -> bool {
        match self {
            Releases::None => false,
            Releases::BeyondText => !is_text_key(code),
            Releases::All => true,
        }
    }
}

fn is_text_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Char(_) | KeyCode::Enter | KeyCode::Tab | KeyCode::Backspace
    )
}

struct HeldKey {
    code: KeyCode,
    seen: f32,
    repeated: bool,
}

pub struct HeldKeys {
    releases: Releases,
    clock: f32,
    tick_secs: f32,
    first_repeat: f32,
    repeat: f32,
    keys: Vec<HeldKey>,
}

impl Default for HeldKeys {
    fn default() -> Self {
        Self {
            releases: Releases::None,
            clock: 0.0,
            tick_secs: 0.0,
            first_repeat: UNLEARNED_FIRST_REPEAT_SECS,
            repeat: UNLEARNED_REPEAT_SECS,
            keys: Vec::new(),
        }
    }
}

impl HeldKeys {
    pub fn hear(&mut self, event: &Event) {
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Release
        {
            self.expect(Releases::heard_from(key.code));
        }
    }

    pub fn expect(&mut self, releases: Releases) {
        self.releases = self.releases.max(releases);
    }

    pub fn reports_release_of(&self, code: KeyCode) -> bool {
        self.releases.reported_for(code)
    }

    pub fn press(&mut self, code: KeyCode) -> Vec<KeyCode> {
        let lifted = self.lift_the_silent_but(code);
        let now = self.clock;
        let Some(key) = self.keys.iter_mut().find(|key| key.code == code) else {
            self.keys.push(HeldKey {
                code,
                seen: now,
                repeated: false,
            });
            return lifted;
        };
        let gap = now - key.seen;
        if key.repeated {
            self.repeat = gap.min(MAX_REPEAT_SECS);
        } else {
            self.first_repeat = gap.min(MAX_FIRST_REPEAT_SECS);
        }
        key.seen = now;
        key.repeated = true;
        lifted
    }

    fn lift_the_silent_but(&mut self, code: KeyCode) -> Vec<KeyCode> {
        let releases = self.releases;
        let (kept, lifted): (Vec<HeldKey>, Vec<HeldKey>) = std::mem::take(&mut self.keys)
            .into_iter()
            .partition(|key| key.code == code || releases.reported_for(key.code));
        self.keys = kept;
        lifted.into_iter().map(|key| key.code).collect()
    }

    pub fn release(&mut self, code: KeyCode) -> bool {
        let before = self.keys.len();
        self.keys.retain(|key| key.code != code);
        self.keys.len() != before
    }

    pub fn is_down(&self, code: KeyCode) -> bool {
        self.keys.iter().any(|key| key.code == code)
    }

    pub fn is_certainly_down(&self, code: KeyCode) -> bool {
        self.keys.iter().any(|key| {
            key.code == code
                && (self.releases.reported_for(code)
                    || self.clock - key.seen <= self.repeat_window())
        })
    }

    pub fn tick(&mut self, dt: f32) -> Vec<KeyCode> {
        self.clock += dt;
        self.tick_secs = dt;
        let (kept, lifted): (Vec<HeldKey>, Vec<HeldKey>) =
            std::mem::take(&mut self.keys).into_iter().partition(|key| {
                self.releases.reported_for(key.code) || self.clock - key.seen <= self.window(key)
            });
        self.keys = kept;
        lifted.into_iter().map(|key| key.code).collect()
    }

    pub fn let_go(&mut self) -> Vec<KeyCode> {
        std::mem::take(&mut self.keys)
            .into_iter()
            .map(|key| key.code)
            .collect()
    }

    fn window(&self, key: &HeldKey) -> f32 {
        if key.repeated {
            return self.repeat_window();
        }
        self.first_repeat * REPEAT_SLACK + STAMP_TICKS * self.tick_secs
    }

    fn repeat_window(&self) -> f32 {
        self.repeat * REPEAT_SLACK + STAMP_TICKS * self.tick_secs
    }
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

    const TICK: f32 = 1.0 / 30.0;
    const LEARNED_EPSILON: f32 = 1e-4;

    fn ticks(keys: &mut HeldKeys, n: usize) -> Vec<KeyCode> {
        (0..n).flat_map(|_| keys.tick(TICK)).collect()
    }

    fn secs(n: usize) -> f32 {
        n as f32 * TICK
    }

    #[test]
    fn a_terminal_that_sends_releases_holds_a_key_until_its_release() {
        let mut keys = HeldKeys::default();
        keys.hear(&event(KeyCode::Enter, KeyEventKind::Release));
        assert!(keys.reports_release_of(KeyCode::Char(' ')));
        keys.press(KeyCode::Down);
        assert!(keys.press(KeyCode::Left).is_empty(), "two keys are held");
        assert!(ticks(&mut keys, 300).is_empty(), "no repeat is needed");
        assert!(keys.is_certainly_down(KeyCode::Down));
        assert!(keys.is_down(KeyCode::Left));
        assert!(keys.release(KeyCode::Down));
        assert!(!keys.is_down(KeyCode::Down));
    }

    #[test]
    fn an_arrow_release_proves_the_arrows_release_and_not_the_text_keys() {
        let mut keys = HeldKeys::default();
        keys.hear(&event(KeyCode::Up, KeyEventKind::Release));
        assert!(keys.reports_release_of(KeyCode::Down));
        assert!(!keys.reports_release_of(KeyCode::Char(' ')));
        assert!(!keys.reports_release_of(KeyCode::Enter));
        keys.press(KeyCode::Down);
        keys.press(KeyCode::Char(' '));
        assert_eq!(
            keys.press(KeyCode::Left),
            vec![KeyCode::Char(' ')],
            "only a key that cannot say it is up is lifted by another press"
        );
        assert_eq!(ticks(&mut keys, 300), Vec::<KeyCode>::new());
        assert!(keys.is_certainly_down(KeyCode::Down));
        assert!(keys.is_certainly_down(KeyCode::Left));
    }

    #[test]
    fn a_tap_on_a_terminal_that_never_sends_a_release_lets_go_on_its_own() {
        let mut keys = HeldKeys::default();
        keys.hear(&event(KeyCode::Down, KeyEventKind::Press));
        keys.press(KeyCode::Down);
        let window = UNLEARNED_FIRST_REPEAT_SECS * REPEAT_SLACK + STAMP_TICKS * TICK;
        let held = (window / TICK) as usize;
        assert!(
            ticks(&mut keys, held).is_empty(),
            "held until a repeat is due"
        );
        assert!(keys.is_down(KeyCode::Down));
        assert_eq!(ticks(&mut keys, 2), vec![KeyCode::Down]);
        assert!(!keys.is_down(KeyCode::Down));
    }

    #[test]
    fn a_press_with_no_repeat_is_certainly_down_only_for_one_repeat() {
        let mut keys = HeldKeys::default();
        keys.press(KeyCode::Down);
        ticks(&mut keys, 1);
        assert!(keys.is_certainly_down(KeyCode::Down), "just pressed");
        let certain = UNLEARNED_REPEAT_SECS * REPEAT_SLACK + STAMP_TICKS * TICK;
        ticks(&mut keys, (certain / TICK) as usize + 1);
        assert!(keys.is_down(KeyCode::Down), "it may still be held");
        assert!(
            !keys.is_certainly_down(KeyCode::Down),
            "but nothing has proved it"
        );
        keys.press(KeyCode::Down);
        assert!(keys.is_certainly_down(KeyCode::Down), "a repeat proves it");
    }

    #[test]
    fn a_key_held_through_its_repeats_stays_down_and_lets_go_soon_after_the_last() {
        const FIRST_REPEAT: usize = 11;
        const REPEAT: usize = 3;
        let mut keys = HeldKeys::default();
        keys.press(KeyCode::Left);
        for tick in 1..=120 {
            keys.tick(TICK);
            let since_first = tick as isize - FIRST_REPEAT as isize;
            if since_first >= 0 && (since_first as usize).is_multiple_of(REPEAT) {
                assert!(
                    keys.press(KeyCode::Left).is_empty(),
                    "a repeat lifts nothing"
                );
            }
            assert!(keys.is_down(KeyCode::Left), "tick {tick}: still held");
            if since_first >= 0 {
                assert!(keys.is_certainly_down(KeyCode::Left), "tick {tick}");
            }
        }
        assert!((keys.first_repeat - secs(FIRST_REPEAT)).abs() < LEARNED_EPSILON);
        assert!((keys.repeat - secs(REPEAT)).abs() < LEARNED_EPSILON);
        let lag = (secs(REPEAT) * REPEAT_SLACK + STAMP_TICKS * TICK) / TICK;
        let lifted = ticks(&mut keys, lag.ceil() as usize + 1);
        assert_eq!(lifted, vec![KeyCode::Left], "let go within {lag} ticks");
    }

    #[test]
    fn a_terminal_that_never_sends_a_release_holds_only_the_last_key_pressed() {
        let mut keys = HeldKeys::default();
        assert!(!keys.reports_release_of(KeyCode::Down));
        keys.press(KeyCode::Down);
        assert_eq!(keys.press(KeyCode::Right), vec![KeyCode::Down]);
        assert!(!keys.is_down(KeyCode::Down), "its repeats stopped for good");
        assert!(keys.is_down(KeyCode::Right));
    }

    #[test]
    fn letting_go_empties_the_hand() {
        let mut keys = HeldKeys::default();
        keys.expect(Releases::All);
        keys.press(KeyCode::Left);
        keys.press(KeyCode::Down);
        assert_eq!(keys.let_go(), vec![KeyCode::Left, KeyCode::Down]);
        assert!(!keys.is_down(KeyCode::Left));
        assert!(!keys.release(KeyCode::Down), "nothing is left to release");
    }

    #[test]
    fn classify_takes_a_press_and_a_repeat_and_never_a_release() {
        assert!(classify(&event(KeyCode::Char('x'), KeyEventKind::Release)).is_none());
        assert!(classify(&event(KeyCode::Char('x'), KeyEventKind::Repeat)).is_some());
        assert!(classify(&event(KeyCode::Char('x'), KeyEventKind::Press)).is_some());
    }
}
