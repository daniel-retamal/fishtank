use std::fs;
use std::path::Path;

use fishtank::fishes::species::FishSpecies;
use fishtank::loot::ConsumableKind;
use fishtank::tank::TankKind;
use fishtank::testing::{Cue, Screenplay, TerminalSize, play_name, plays_in, review, stage};

const REEL_DIR: &str = env!("CARGO_TARGET_TMPDIR");
const JOURNEY_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/journeys");
const SCENE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/journeys/scenes");
const SCENE_INCLUDE: &str = "include scenes/";
const GOD_COMMANDS: [&str; 10] = [
    "/spawn",
    "/give",
    "/add",
    "/subtract",
    "/mutate",
    "/revive",
    "/clone",
    "/expand",
    "/restore",
    "/bless",
];
const TAKING_COMMANDS: [&str; 2] = ["/give", "/add"];
const STACKABLE_SHELVES: [&str; 3] = ["food", "coffee", "bait"];
const JOURNEY_REEL_PREFIX: &str = "journey";
const JOURNEY_SIZES: [TerminalSize; 4] = [
    TerminalSize::new(100, 30),
    TerminalSize::new(80, 24),
    TerminalSize::new(60, 18),
    TerminalSize::new(40, 14),
];
const SMALLEST_SUPPORTED: TerminalSize = TerminalSize::new(40, 14);

#[test]
fn every_critical_user_journey_completes_at_every_supported_size() {
    let journeys = plays_in(Path::new(JOURNEY_DIR));
    assert!(
        !journeys.is_empty(),
        "tests/journeys holds the critical user journeys"
    );
    let mut failures = Vec::new();
    for journey in &journeys {
        for size in JOURNEY_SIZES {
            let reel_name = format!(
                "{JOURNEY_REEL_PREFIX}-{}-{}",
                play_name(journey),
                size.label()
            );
            let performance = stage(journey, size, Path::new(REEL_DIR), &reel_name);
            failures.extend(review(performance, &reel_name));
        }
    }
    assert!(
        failures.is_empty(),
        "open {REEL_DIR}/reels/{JOURNEY_REEL_PREFIX}-*.txt to see the screens:\n{}",
        failures.join("\n")
    );
}

#[test]
fn the_smallest_size_every_journey_runs_at_is_a_real_small_screen() {
    assert_eq!(
        JOURNEY_SIZES.iter().min_by_key(|size| size.cols),
        Some(&SMALLEST_SUPPORTED),
        "a journey that only passes on a big terminal does not prove the game is responsive"
    );
}

#[test]
fn no_journey_sets_itself_up_with_a_god_command() {
    let mut offences = Vec::new();
    for journey in plays_in(Path::new(JOURNEY_DIR)) {
        let play = Screenplay::load(&journey).unwrap_or_else(|error| panic!("{error}"));
        for cue in play.cues() {
            let (Cue::Run(line) | Cue::Type(line)) = cue else {
                continue;
            };
            let command = line.split_whitespace().next().unwrap_or_default();
            if !GOD_COMMANDS.contains(&command.to_ascii_lowercase().as_str()) {
                continue;
            }
            if takes_what_no_shop_sells(&command.to_ascii_lowercase(), line) {
                continue;
            }
            offences.push(format!("{}: {line}", play_name(&journey)));
        }
    }
    assert!(
        offences.is_empty(),
        "a journey that sets itself up with a god command proves nothing; buy it through the shop:
{}",
        offences.join(
            "
"
        )
    );
}

fn shop_sells(thing: &str) -> bool {
    let named = thing.trim().to_ascii_lowercase();
    if named.is_empty() {
        return false;
    }
    STACKABLE_SHELVES.contains(&named.as_str())
        || FishSpecies::all_buyable()
            .iter()
            .any(|species| species.display_name().eq_ignore_ascii_case(&named))
        || TankKind::all_buyable()
            .iter()
            .any(|kind| kind.display_name().eq_ignore_ascii_case(&named))
        || ConsumableKind::bench_stock()
            .iter()
            .any(|kind| kind.display_name().eq_ignore_ascii_case(&named))
}

fn takes_what_no_shop_sells(command: &str, line: &str) -> bool {
    if !TAKING_COMMANDS.contains(&command) {
        return false;
    }
    let rest = line[command.len()..].trim();
    let named = rest
        .rsplit_once(' ')
        .filter(|(_, tail)| tail.parse::<u32>().is_ok())
        .map_or(rest, |(head, _)| head);
    !shop_sells(named)
}

#[test]
fn a_journey_may_take_only_what_no_shop_sells() {
    assert!(
        takes_what_no_shop_sells("/give", "/give computer"),
        "a Computer is found, never bought, so a journey may start with one"
    );
    assert!(
        !takes_what_no_shop_sells("/give", "/give inverter coil"),
        "the bench sells coils, so a journey buys them"
    );
    assert!(
        !takes_what_no_shop_sells("/add", "/add bait 2"),
        "the shop sells bait by the counter"
    );
    assert!(
        !takes_what_no_shop_sells("/spawn", "/spawn botfish \"Neo\""),
        "spawning is never a way to obtain anything"
    );
}

#[test]
fn every_scene_is_performed_by_some_journey() {
    let journeys: Vec<String> = plays_in(Path::new(JOURNEY_DIR))
        .iter()
        .map(|journey| fs::read_to_string(journey).expect("a journey is readable"))
        .collect();
    let scenes = plays_in(Path::new(SCENE_DIR));
    assert!(
        !scenes.is_empty(),
        "tests/journeys/scenes holds the shared openings"
    );
    for scene in scenes {
        let cue = format!("{SCENE_INCLUDE}{}.play", play_name(&scene));
        assert!(
            journeys.iter().any(|journey| journey.contains(&cue)),
            "no journey performs {cue:?}"
        );
    }
}
