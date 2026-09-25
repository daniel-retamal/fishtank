use std::collections::BTreeSet;

use crate::fishes::parts::KeyBinding;
use crate::ui::hint_bar::HintBar;
use crate::ui::hints::HINT_ESC_LEAVE;

const CONSOLE_LABEL: &str = "console";
const ARROWS: &[(&str, &str)] = &[("up", "↑"), ("down", "↓"), ("left", "←"), ("right", "→")];

pub struct ConsoleState {
    pub fish: String,
    down: BTreeSet<String>,
    fresh: BTreeSet<String>,
    lifted: BTreeSet<String>,
}

impl ConsoleState {
    pub fn new(fish: String) -> Self {
        Self {
            fish,
            down: BTreeSet::new(),
            fresh: BTreeSet::new(),
            lifted: BTreeSet::new(),
        }
    }

    pub fn press(&mut self, key: &str) -> bool {
        self.lifted.remove(key);
        self.fresh.insert(key.to_string());
        self.down.insert(key.to_string())
    }

    pub fn release(&mut self, key: &str) -> bool {
        if !self.down.contains(key) {
            return false;
        }
        if self.fresh.contains(key) {
            self.lifted.insert(key.to_string());
            return false;
        }
        self.down.remove(key)
    }

    pub fn settle(&mut self) -> Vec<String> {
        self.fresh.clear();
        let lifted = std::mem::take(&mut self.lifted);
        for key in &lifted {
            self.down.remove(key);
        }
        lifted.into_iter().collect()
    }

    pub fn leave(self) -> Vec<String> {
        self.down.into_iter().collect()
    }

    pub fn is_down(&self, key: &str) -> bool {
        self.down.contains(key)
    }

    pub fn hints(&self, bindings: &[KeyBinding]) -> HintBar {
        bindings.iter().fold(
            HintBar::new(HINT_ESC_LEAVE).action(format!("{CONSOLE_LABEL} {}", self.fish)),
            |bar, binding| {
                bar.level(
                    format!("{} {}", key_label(&binding.key), binding.pin),
                    self.is_down(&binding.key),
                )
            },
        )
    }
}

pub fn key_label(key: &str) -> String {
    if let Some(&(_, arrow)) = ARROWS.iter().find(|(name, _)| *name == key) {
        return arrow.to_string();
    }
    if key.chars().count() == 1 {
        return key.to_string();
    }
    key.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn console() -> ConsoleState {
        ConsoleState::new("Pad".to_string())
    }

    #[test]
    fn a_held_key_stays_down_across_ticks_until_it_is_released() {
        let mut console = console();
        assert!(console.press("up"));
        assert!(console.settle().is_empty(), "nothing was let go");
        assert!(console.is_down("up"));
        assert!(console.release("up"), "an old press lets go at once");
        assert!(!console.is_down("up"));
    }

    #[test]
    fn a_tap_between_two_ticks_is_held_until_the_next_tick_has_seen_it() {
        let mut console = console();
        console.press("space");
        assert!(!console.release("space"), "the fabric has not seen it yet");
        assert!(console.is_down("space"));
        assert_eq!(console.settle(), vec!["space".to_string()]);
        assert!(!console.is_down("space"));
    }

    #[test]
    fn a_key_pressed_again_before_the_tick_is_not_let_go() {
        let mut console = console();
        console.press("space");
        console.release("space");
        console.press("space");
        assert!(console.settle().is_empty());
        assert!(console.is_down("space"));
    }

    #[test]
    fn releasing_a_key_the_console_never_pressed_changes_nothing() {
        let mut console = console();
        assert!(
            !console.release("enter"),
            "the Enter that opened the console"
        );
        assert!(console.settle().is_empty());
    }

    #[test]
    fn a_repeated_press_is_one_key_held() {
        let mut console = console();
        assert!(console.press("up"));
        assert!(!console.press("up"), "a repeat changes nothing");
        console.settle();
        assert!(console.release("up"));
    }

    #[test]
    fn leaving_lets_go_of_every_key_still_down() {
        let mut console = console();
        console.press("up");
        console.press("space");
        console.release("space");
        let mut left = console.leave();
        left.sort();
        assert_eq!(left, vec!["space".to_string(), "up".to_string()]);
    }

    #[test]
    fn a_key_is_labelled_the_way_every_hint_names_it() {
        assert_eq!(key_label("up"), "↑");
        assert_eq!(key_label("right"), "→");
        assert_eq!(key_label("space"), "SPACE");
        assert_eq!(key_label("enter"), "ENTER");
        assert_eq!(key_label("w"), "w");
    }
}
