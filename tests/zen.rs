use std::path::Path;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use fishtank::testing::Tui;

const COLS: u16 = 100;
const ROWS: u16 = 30;
const STATS: &str = "cash:";
const TANK_NAME: &str = "Fishtank";
const NAME: &str = "Zazen";
const COW_LINE: &str = "om";

fn zen_lab() -> Tui {
    let mut tui = Tui::with_size(COLS, ROWS);
    tui.film(Path::new(env!("CARGO_TARGET_TMPDIR")), "zen");
    tui.clear_tank();
    tui.run(&format!("/spawn merluza \"{NAME}\""));
    for fish in &mut tui.app.tanks[0].fish {
        fish.frozen = true;
        fish.position.y = f32::from(ROWS / 2);
    }
    tui.run("/names");
    tui.run("/nets");
    tui
}

fn event(tui: &mut Tui, event: Event) {
    tui.app.handle_input(event);
    tui.tick_n(1);
}

fn ctrl_c() -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
}

#[test]
fn zen_leaves_only_the_tank_and_its_fishes() {
    let mut tui = zen_lab();
    tui.screen().expect_find(NAME);
    tui.screen().expect_find(STATS);
    tui.snap("before: names, nets and both bars");

    tui.run("/zen");
    tui.snap("/zen: the tank alone");

    assert!(tui.app.in_zen());
    let screen = tui.screen();
    screen.expect_absent(NAME);
    screen.expect_absent(STATS);
    assert!(
        !screen
            .rows()
            .iter()
            .any(|row| row.starts_with('>') && row[1..].trim().is_empty()),
        "the command prompt is gone"
    );
    assert_eq!(
        tui.app.tanks[0].height, ROWS,
        "the tank grows over the rows the bars gave up"
    );
    assert!(
        tui.app.settings.show_names && tui.app.settings.show_nets && tui.app.settings.show_stats,
        "zen hides the labels without touching the player's toggles"
    );

    tui.key(KeyCode::Esc);
    tui.screen().expect_find(NAME);
    tui.screen().expect_find(STATS);
}

#[test]
fn a_speech_bubble_still_speaks_in_zen() {
    let mut tui = zen_lab();
    tui.run("/give cow");
    tui.run(&format!("/cowsay \"{COW_LINE}\""));
    let said = tui.app.tanks[0].cows[0]
        .speech
        .as_ref()
        .map(|bubble| bubble.text.clone())
        .expect("the cow spoke");
    tui.run("/zen");
    tui.snap("a cow speaks in zen");
    tui.screen().expect_find(&format!("< {said} >"));
}

#[test]
fn any_key_press_leaves_zen_and_types_nothing() {
    let mut tui = zen_lab();
    tui.run("/zen");

    tui.key(KeyCode::Char('x'));
    assert!(!tui.app.in_zen(), "a key press wakes the tank");
    assert_eq!(tui.app.editor.text, "", "the waking key is not typed");
    tui.screen().expect_find(STATS);
    tui.snap("any key: back to the bars");

    tui.key(KeyCode::Char('x'));
    assert_eq!(tui.app.editor.text, "x", "the next key types as usual");
}

#[test]
fn every_way_out_works_the_same() {
    for key in [
        KeyCode::Esc,
        KeyCode::Enter,
        KeyCode::Char('q'),
        KeyCode::Char(' '),
    ] {
        let mut tui = zen_lab();
        tui.run("/zen");
        tui.key(key);
        assert!(!tui.app.in_zen(), "{key:?} leaves zen");
        assert!(tui.app.running, "{key:?} does not quit the game");
    }
}

#[test]
fn alt_tab_a_release_a_focus_change_or_a_resize_stays_in_zen() {
    let mut tui = zen_lab();
    tui.run("/zen");
    let mut released = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
    released.kind = KeyEventKind::Release;
    let alt = KeyEvent::new(
        KeyCode::Modifier(crossterm::event::ModifierKeyCode::LeftAlt),
        KeyModifiers::ALT,
    );
    for quiet in [
        Event::Key(released),
        Event::Key(alt),
        Event::FocusLost,
        Event::FocusGained,
        Event::Resize(COLS, ROWS),
        Event::Paste("typed elsewhere".to_string()),
    ] {
        event(&mut tui, quiet.clone());
        assert!(tui.app.in_zen(), "{quiet:?} must not wake the tank");
    }
    assert_eq!(tui.app.editor.text, "");
}

#[test]
fn ctrl_c_still_quits_from_zen() {
    let mut tui = zen_lab();
    tui.run("/zen");
    event(&mut tui, ctrl_c());
    assert!(!tui.app.running);
}

#[test]
fn zen_fills_every_size_without_a_broken_border() {
    for (cols, rows) in [(100, 30), (60, 18), (40, 14), (28, 10)] {
        let mut tui = Tui::with_size(cols, rows);
        tui.film(
            Path::new(env!("CARGO_TARGET_TMPDIR")),
            &format!("zen-{cols}x{rows}"),
        );
        tui.run("/zen");
        tui.snap("zen");
        tui.screen().expect_absent(TANK_NAME);
        assert_eq!(tui.app.tanks[tui.app.current_tank].height, rows);
        let report = tui.reel().flaw_report();
        assert!(report.is_empty(), "{cols}×{rows}: {report}");
    }
}
