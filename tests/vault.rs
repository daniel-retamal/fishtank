use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{Event, KeyCode, KeyEvent};
use fishtank::app::{App, Launch, SAVE_VERSION};
use fishtank::fishes::fish::Fish;
use fishtank::fishes::species::FishSpecies;
use fishtank::tanks::soul_wall::SOUL_WALL_LIMIT;
use fishtank::testing::{DEFAULT_COLS, DEFAULT_ROWS, Tui};
use fishtank::vault::{self, Vault, VaultError};

const SCRATCH: &str = env!("CARGO_TARGET_TMPDIR");
const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/saves/fishtank-v1.ron");
const CRYPT_EXTENSION: &str = "crypt";
const DOOMED: &str = "Doomed";
const HONEST_FOOD: &str = "food_supply: 60,";
const FORGED_FOOD: &str = "food_supply: 6000,";
const NOTICE_TITLE: &str = "Save#aside";
const UNSAVED: &str = "unsaved";
const NOTICE_SIZES: [(u16, u16); 4] = [(100, 30), (60, 18), (40, 14), (28, 10)];

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(SCRATCH).join("vault").join(test);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

fn save_path(dir: &Path, launch: Launch) -> PathBuf {
    Vault::open_in(dir.to_path_buf(), launch)
        .expect("a vault")
        .path()
}

fn opened(dir: &Path, launch: Launch) -> App {
    let vault = Vault::open_in(dir.to_path_buf(), launch).expect("a vault");
    App::open(launch, vault, DEFAULT_COLS, DEFAULT_ROWS).expect("a game")
}

fn enter(app: &mut App, line: &str) {
    app.editor.set(line.to_string());
    app.handle_input(Event::Key(KeyEvent::from(KeyCode::Enter)));
}

fn bury(app: &mut App, prefix: &str, count: usize) {
    for n in 0..count {
        enter(app, &format!("/spawn goldfish \"{prefix}{n}\""));
        enter(app, &format!("/kill \"{prefix}{n}\""));
    }
}

fn written(app: &mut App) {
    app.persist_and_wait().expect("the scribe wrote it down");
}

fn crypts(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .expect("a directory")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == CRYPT_EXTENSION))
        .collect();
    found.sort();
    found
}

fn the_crypt(dir: &Path) -> PathBuf {
    let found = crypts(dir);
    assert_eq!(found.len(), 1, "one crypt, found {found:?}");
    found[0].clone()
}

fn dead_names(app: &App) -> Vec<String> {
    app.graveyard.iter().map(|fish| fish.name.clone()).collect()
}

fn holds_a_cheatfish(app: &App) -> bool {
    app.tanks
        .iter()
        .flat_map(|tank| tank.fish.iter())
        .any(|fish| fish.species == FishSpecies::Cheatfish)
}

fn swims(app: &App, name: &str) -> bool {
    app.tanks
        .iter()
        .flat_map(|tank| tank.fish.iter())
        .any(|fish| fish.name == name)
}

fn graves(dir: &Path) -> usize {
    fs::read_to_string(the_crypt(dir))
        .expect("a crypt")
        .lines()
        .count()
}

#[test]
fn the_dead_are_written_into_the_crypt_once_and_never_again() {
    let dir = scratch("once");
    let save = save_path(&dir, Launch::Debug);
    let mut app = opened(&dir, Launch::Debug);
    bury(&mut app, DOOMED, 3);
    written(&mut app);
    let crypt = the_crypt(&dir);
    let first = fs::read(&crypt).unwrap();

    written(&mut app);
    bury(&mut app, "Later", 1);
    written(&mut app);

    assert_eq!(the_crypt(&dir), crypt, "a burial appends to the same crypt");
    let now = fs::read(&crypt).unwrap();
    assert!(
        now.starts_with(&first),
        "nothing already buried is rewritten"
    );
    assert_eq!(graves(&dir), 4);
    assert!(
        !fs::read_to_string(save)
            .unwrap()
            .contains(&format!("name: \"{DOOMED}")),
        "the save names no dead fish"
    );
}

#[test]
fn every_fish_that_ever_died_comes_back_and_the_oldest_can_still_be_revived() {
    let dir = scratch("every-grave");
    let many = SOUL_WALL_LIMIT * 2;
    let before = {
        let mut app = opened(&dir, Launch::Debug);
        bury(&mut app, DOOMED, many);
        written(&mut app);
        dead_names(&app)
    };

    let mut app = opened(&dir, Launch::Debug);

    assert_eq!(dead_names(&app), before);
    enter(&mut app, &format!("/revive \"{DOOMED}0\""));
    assert!(swims(&app, &format!("{DOOMED}0")), "the very first grave");
}

#[test]
fn a_revived_grave_stays_out_of_the_crypt_after_a_restart() {
    let dir = scratch("revived");
    {
        let mut app = opened(&dir, Launch::Debug);
        bury(&mut app, DOOMED, 5);
        written(&mut app);
        enter(&mut app, &format!("/revive \"{DOOMED}1\""));
        bury(&mut app, "Later", 1);
        written(&mut app);
    }

    let app = opened(&dir, Launch::Debug);

    assert_eq!(
        dead_names(&app),
        ["Doomed0", "Doomed2", "Doomed3", "Doomed4", "Later0"]
    );
    assert!(swims(&app, "Doomed1"));
    assert_eq!(crypts(&dir).len(), 1, "the old crypt is swept away");
}

#[test]
fn a_crash_between_the_crypt_and_the_save_loses_nothing_and_mints_nothing() {
    let dir = scratch("crash");
    let save = save_path(&dir, Launch::Debug);
    let before_the_crash = {
        let mut app = opened(&dir, Launch::Debug);
        bury(&mut app, DOOMED, 2);
        enter(&mut app, "/spawn goldfish \"Lucky\"");
        written(&mut app);
        let before = fs::read(&save).unwrap();
        enter(&mut app, "/kill \"Lucky\"");
        written(&mut app);
        before
    };
    assert_eq!(graves(&dir), 3, "the crypt got its line");
    fs::write(&save, before_the_crash).unwrap();

    let opened_again = vault::read_save(&save).expect("a save");
    assert!(opened_again.intact, "a crash is not a forgery");
    let mut app = opened(&dir, Launch::Debug);

    assert_eq!(dead_names(&app), ["Doomed0", "Doomed1"]);
    assert!(swims(&app, "Lucky"), "the save never saw Lucky die");
    bury(&mut app, "Later", 1);
    written(&mut app);
    assert_eq!(
        graves(&dir),
        3,
        "the stray line was cut before the next grave"
    );
}

#[test]
fn an_honest_save_mints_nothing() {
    let dir = scratch("honest");
    written(&mut opened(&dir, Launch::Player));

    let app = opened(&dir, Launch::Player);

    assert!(!holds_a_cheatfish(&app));
}

#[test]
fn a_hand_edited_save_still_loads_and_leaves_a_cheatfish() {
    let dir = scratch("forged");
    let save = save_path(&dir, Launch::Player);
    written(&mut opened(&dir, Launch::Player));
    let text = fs::read_to_string(&save).unwrap();
    assert!(text.contains(HONEST_FOOD));
    fs::write(&save, text.replacen(HONEST_FOOD, FORGED_FOOD, 1)).unwrap();

    let app = opened(&dir, Launch::Player);

    assert_eq!(app.food_supply, 6000, "the edit is honoured");
    assert!(holds_a_cheatfish(&app), "and marked");
}

#[test]
fn an_edited_grave_is_a_forged_save_too() {
    let dir = scratch("forged-grave");
    {
        let mut app = opened(&dir, Launch::Player);
        app.graveyard.push(Fish::new(
            FishSpecies::Koi,
            DOOMED.to_string(),
            0.0,
            0.0,
            &mut rand::rng(),
        ));
        written(&mut app);
    }
    let crypt = the_crypt(&dir);
    let text = fs::read_to_string(&crypt).unwrap();
    fs::write(&crypt, text.replacen("Koi", "Cheatfish", 1)).unwrap();

    let app = opened(&dir, Launch::Player);

    assert!(holds_a_cheatfish(&app));
}

#[test]
fn a_save_written_before_the_seal_counts_as_forged() {
    let dir = scratch("unsealed");
    let save = save_path(&dir, Launch::Player);
    fs::copy(FIXTURE, &save).unwrap();

    let opened = vault::read_save(&save).expect("it still loads");

    assert!(!opened.intact);
    assert!(opened.save.debug_mode(), "a lab bench, so it mints nothing");
}

#[test]
fn a_save_that_cannot_be_read_is_never_touched() {
    let dir = scratch("unreachable");
    let save = save_path(&dir, Launch::Player);
    fs::create_dir_all(&save).unwrap();
    let vault = Vault::open_in(dir, Launch::Player).expect("a vault");

    let refused = App::open(Launch::Player, vault, DEFAULT_COLS, DEFAULT_ROWS);

    assert!(matches!(refused, Err(VaultError::Unreachable { .. })));
    assert!(save.is_dir(), "left exactly as it was");
}

#[test]
fn a_save_from_a_newer_game_refuses_to_start_and_is_left_alone() {
    let dir = scratch("future");
    let save = save_path(&dir, Launch::Player);
    let text = App::new().snapshot().to_ron().unwrap().replacen(
        &format!("version: {SAVE_VERSION},"),
        "version: 999,",
        1,
    );
    fs::write(&save, &text).unwrap();
    let vault = Vault::open_in(dir, Launch::Player).expect("a vault");

    let refused = App::open(Launch::Player, vault, DEFAULT_COLS, DEFAULT_ROWS);

    assert!(matches!(
        refused,
        Err(VaultError::FromTheFuture { version: 999, .. })
    ));
    assert_eq!(fs::read_to_string(&save).unwrap(), text);
}

#[test]
fn a_newer_save_this_game_cannot_even_parse_is_still_known_to_be_from_the_future() {
    let dir = scratch("future-unparseable");
    let save = save_path(&dir, Launch::Player);
    let text = App::new()
        .snapshot()
        .to_ron()
        .unwrap()
        .replacen(&format!("version: {SAVE_VERSION},"), "version: 999,", 1)
        .replacen("Goldfish", "Hyperfish", 1);
    fs::write(&save, &text).unwrap();
    let vault = Vault::open_in(dir, Launch::Player).expect("a vault");

    let refused = App::open(Launch::Player, vault, DEFAULT_COLS, DEFAULT_ROWS);

    assert!(matches!(refused, Err(VaultError::FromTheFuture { .. })));
    assert_eq!(fs::read_to_string(&save).unwrap(), text);
}

#[cfg(windows)]
#[test]
fn a_garbled_save_that_cannot_be_moved_aside_refuses_to_start() {
    use std::os::windows::fs::OpenOptionsExt;

    let dir = scratch("stuck");
    let save = save_path(&dir, Launch::Player);
    fs::write(&save, "not a fishtank").unwrap();
    let _held = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&save)
        .unwrap();
    let vault = Vault::open_in(dir, Launch::Player).expect("a vault");

    let refused = App::open(Launch::Player, vault, DEFAULT_COLS, DEFAULT_ROWS);

    assert!(matches!(
        refused,
        Err(VaultError::Unreachable { .. } | VaultError::Stuck { .. })
    ));
}

#[test]
fn a_garbled_save_is_set_aside_and_announced_at_every_size() {
    for (cols, rows) in NOTICE_SIZES {
        let dir = scratch(&format!("garbled-{cols}x{rows}"));
        fs::write(save_path(&dir, Launch::Player), "not a fishtank").unwrap();
        let vault = Vault::open_in(dir.clone(), Launch::Player).expect("a vault");
        let app = App::open(Launch::Player, vault, cols, rows).expect("a new game");
        let mut tui = Tui::around(app, cols, rows);
        tui.film(Path::new(SCRATCH), &format!("save-aside-{cols}x{rows}"));

        tui.snap("the notice over a new game");
        tui.screen().expect_find(NOTICE_TITLE);
        tui.key(KeyCode::Down).key(KeyCode::Down);
        tui.snap("scrolled to the path");
        tui.key(KeyCode::Esc);
        tui.snap("closed");

        tui.screen().expect_absent(NOTICE_TITLE);
        assert!(
            tui.reel().flaw_report().is_empty(),
            "{cols}x{rows}: {}",
            tui.reel().flaw_report()
        );
        let kept = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().contains("unreadable"))
            .count();
        assert_eq!(kept, 1, "the garbled save is kept");
    }
}

#[test]
fn a_save_that_cannot_be_written_shows_unsaved_until_it_can() {
    let dir = scratch("unwritable");
    let save = save_path(&dir, Launch::Player);
    let app = opened(&dir, Launch::Player);
    let mut tui = Tui::around(app, DEFAULT_COLS, DEFAULT_ROWS);
    fs::create_dir_all(save.join("in-the-way")).unwrap();

    assert!(tui.app.persist_and_wait().is_err());
    tui.screen().expect_find(UNSAVED);

    fs::remove_dir_all(&save).unwrap();
    assert!(tui.app.persist_and_wait().is_ok());
    tui.screen().expect_absent(UNSAVED);
}
