use std::path::Path;

use crossterm::event::KeyCode;
use fishtank::testing::Tui;

const CAST_TICKS: usize = 30 * 90;
const CASTS: usize = 6;
const LOOKAHEAD_STEPS: f32 = 6.0;
const DEADBAND: f32 = 0.04;
const REEL_ZONE: f32 = 0.3;
const CENTRE: f32 = 0.5;
const CATCH_CARD_HINTS: [&str; 2] = ["ENTER capture", "ESC/q close"];
const LATCHED_REEL_HINT: &str = "↑ stop";
const HELD_REEL_HINT: &str = "↓ reel";
const TWO_SECONDS: usize = 60;
const WHOLE_HINT_COLS: u16 = 80;
const FILM_SIZES: [(u16, u16); 5] = [(100, 30), (80, 24), (60, 18), (40, 14), (28, 10)];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Terminal {
    Releasing,
    PressOnly,
}

#[derive(Clone, Copy, Debug)]
struct Cadence {
    first_repeat: u32,
    repeat: u32,
}

const WINDOWS_CADENCE: Cadence = Cadence {
    first_repeat: 15,
    repeat: 1,
};

const PRESS_ONLY_CADENCES: [Cadence; 4] = [
    Cadence {
        first_repeat: 7,
        repeat: 1,
    },
    Cadence {
        first_repeat: 11,
        repeat: 3,
    },
    Cadence {
        first_repeat: 20,
        repeat: 1,
    },
    Cadence {
        first_repeat: 31,
        repeat: 5,
    },
];

struct Hand {
    terminal: Terminal,
    cadence: Cadence,
    held: Vec<(KeyCode, u32)>,
    reeling: bool,
}

impl Hand {
    fn new(terminal: Terminal, cadence: Cadence) -> Self {
        Self {
            terminal,
            cadence,
            held: Vec::new(),
            reeling: false,
        }
    }

    fn run(&mut self, tui: &mut Tui, line: &str) {
        tui.run(line);
        if self.terminal == Terminal::Releasing {
            tui.release(KeyCode::Enter);
        }
    }

    fn hold(&mut self, tui: &mut Tui, code: KeyCode, down: bool) {
        let at = self.held.iter().position(|(held, _)| *held == code);
        match (at, down) {
            (None, true) => {
                tui.key(code);
                self.held.push((code, 0));
            }
            (Some(i), false) => {
                self.held.remove(i);
                if self.terminal == Terminal::Releasing {
                    tui.release(code);
                }
            }
            _ => {}
        }
    }

    fn reel(&mut self, tui: &mut Tui, want: bool) {
        if self.terminal == Terminal::Releasing {
            self.hold(tui, KeyCode::Down, want);
            return;
        }
        if want == self.reeling {
            return;
        }
        self.reeling = want;
        self.held.clear();
        tui.key(if want { KeyCode::Down } else { KeyCode::Up });
    }

    fn let_go(&mut self, tui: &mut Tui) {
        self.reeling = false;
        for (code, _) in std::mem::take(&mut self.held) {
            if self.terminal == Terminal::Releasing {
                tui.release(code);
            }
        }
    }

    fn repeats_on(&self, ticks: u32) -> bool {
        ticks >= self.cadence.first_repeat
            && (ticks - self.cadence.first_repeat).is_multiple_of(self.cadence.repeat)
    }

    fn repeating(&self) -> &[(KeyCode, u32)] {
        match self.terminal {
            Terminal::Releasing => &self.held,
            Terminal::PressOnly => {
                let last = self.held.len().saturating_sub(1);
                &self.held[last..]
            }
        }
    }

    fn tick(&mut self, tui: &mut Tui) {
        for (_, ticks) in &mut self.held {
            *ticks += 1;
        }
        let due: Vec<KeyCode> = self
            .repeating()
            .iter()
            .filter(|(_, ticks)| self.repeats_on(*ticks))
            .map(|(code, _)| *code)
            .collect();
        for code in due {
            tui.key(code);
        }
        tui.tick_n(1);
    }
}

fn card_shown(tui: &mut Tui) -> bool {
    let screen = tui.screen();
    CATCH_CARD_HINTS.iter().any(|hint| screen.contains(hint))
}

fn reel_like_a_player(tui: &mut Tui, hand: &mut Hand) -> bool {
    hand.run(tui, "/fish");
    for _ in 0..CAST_TICKS {
        let Some(state) = tui.app.fishing_state() else {
            hand.let_go(tui);
            return card_shown(tui);
        };
        if state.is_catching() {
            if state.is_biting() {
                hand.reel(tui, true);
            }
            hand.tick(tui);
            continue;
        }
        let ahead = state.fish_pos + state.fish_velocity * LOOKAHEAD_STEPS;
        let reel = (ahead - CENTRE).abs() * 2.0 < REEL_ZONE;
        hand.reel(tui, reel);
        hand.hold(tui, KeyCode::Left, ahead > CENTRE + DEADBAND);
        hand.hold(tui, KeyCode::Right, ahead < CENTRE - DEADBAND);
        hand.tick(tui);
    }
    panic!("a cast outlasted {CAST_TICKS} ticks");
}

fn landed_casts(terminal: Terminal, cadence: Cadence) -> usize {
    let mut tui = Tui::new();
    tui.clear_tank();
    let mut hand = Hand::new(terminal, cadence);
    let mut landed = 0;
    for cast in 0..CASTS {
        if !reel_like_a_player(&mut tui, &mut hand) {
            continue;
        }
        landed += 1;
        tui.type_text(&format!("Kept {cast}"));
        tui.key(KeyCode::Enter);
        assert!(!card_shown(&mut tui), "the card closed");
    }
    landed
}

fn hook_a_fish(tui: &mut Tui) {
    tui.run("/fish --no-escape");
    for _ in 0..CAST_TICKS {
        if tui.app.fishing_state().is_some_and(|s| s.is_biting()) {
            break;
        }
        tui.tick_n(1);
    }
    tui.key(KeyCode::Down);
    let state = tui.app.fishing_state().expect("the fish is hooked");
    assert!(!state.is_catching(), "↓ on a bite hooks the fish");
    assert!(state.is_reeling, "and starts the reel");
}

fn reeling(tui: &Tui) -> bool {
    tui.app.fishing_state().expect("no escape").is_reeling
}

#[test]
fn a_player_lands_every_cast_through_a_terminal_that_sends_releases() {
    assert_eq!(landed_casts(Terminal::Releasing, WINDOWS_CADENCE), CASTS);
}

#[test]
fn a_player_lands_every_cast_through_a_terminal_that_never_sends_a_release() {
    for cadence in PRESS_ONLY_CADENCES {
        assert_eq!(
            landed_casts(Terminal::PressOnly, cadence),
            CASTS,
            "{cadence:?}"
        );
    }
}

#[test]
fn a_terminal_that_never_sends_a_release_latches_the_reel_until_up() {
    let mut tui = Tui::new();
    tui.clear_tank();
    hook_a_fish(&mut tui);
    tui.screen().expect_find(LATCHED_REEL_HINT);

    tui.tick_n(TWO_SECONDS);
    assert!(reeling(&tui), "the reel holds with no repeat");
    tui.key(KeyCode::Up);
    assert!(!reeling(&tui), "↑ stops it");
    tui.tick_n(TWO_SECONDS);
    assert!(!reeling(&tui), "and it stays stopped");
    tui.key(KeyCode::Down);
    assert!(reeling(&tui), "↓ starts it again");
}

#[test]
fn a_steering_key_with_no_repeat_lets_go_on_its_own() {
    let mut tui = Tui::new();
    tui.clear_tank();
    hook_a_fish(&mut tui);
    tui.key(KeyCode::Right);
    assert!(tui.app.fishing_state().expect("hooked").is_pushing_right);
    tui.tick_n(TWO_SECONDS);
    let state = tui.app.fishing_state().expect("no escape");
    assert!(!state.is_pushing_right, "a key that stops repeating is up");
    assert!(state.is_reeling, "and the reel is not a held key");
}

#[test]
fn a_terminal_that_sends_releases_reels_only_while_down_is_held() {
    let mut tui = Tui::new();
    tui.clear_tank();
    tui.release(KeyCode::Enter);
    hook_a_fish(&mut tui);
    let screen = tui.screen();
    screen.expect_find(HELD_REEL_HINT);
    screen.expect_absent(LATCHED_REEL_HINT);

    tui.tick_n(TWO_SECONDS);
    assert!(reeling(&tui), "held with no repeat");
    tui.release(KeyCode::Down);
    assert!(!reeling(&tui), "let go at once");
}

#[test]
fn the_reel_says_how_to_stop_it_at_every_size() {
    for terminal in [Terminal::Releasing, Terminal::PressOnly] {
        for (cols, rows) in FILM_SIZES {
            let mut tui = Tui::with_size(cols, rows);
            tui.film(
                Path::new(env!("CARGO_TARGET_TMPDIR")),
                &format!("fishing-{terminal:?}-{cols}x{rows}"),
            );
            tui.clear_tank();
            if terminal == Terminal::Releasing {
                tui.release(KeyCode::Enter);
            }
            hook_a_fish(&mut tui);
            tui.tick_n(1);
            tui.snap(&format!("{terminal:?}: the reel and its hints"));
            if cols < WHOLE_HINT_COLS {
                continue;
            }
            let screen = tui.screen();
            screen.expect_find(HELD_REEL_HINT);
            if terminal == Terminal::PressOnly {
                screen.expect_find(LATCHED_REEL_HINT);
            } else {
                screen.expect_absent(LATCHED_REEL_HINT);
            }
        }
    }
}
