use std::path::Path;

use fishtank::cli::{Invocation, NAME};
use fishtank::testing::Tui;
use fishtank::update::{Route, Stamp, UPDATE_LABEL, Watch, release_in, retired};
use semver::Version;

const DAY_SECS: u64 = 24 * 60 * 60;

fn version(text: &str) -> Version {
    Version::parse(text).unwrap()
}

fn headers_to(location: &str) -> String {
    format!(
        "HTTP/2 302 \r\ndate: Thu, 25 Sep 2026 03:00:00 GMT\r\nLocation: {location}\r\ncontent-length: 0\r\n\r\n"
    )
}

#[test]
fn the_latest_release_is_read_from_the_tag_github_redirects_to() {
    let headers = headers_to("https://github.com/daniel-retamal/fishtanks/releases/tag/v1.2.3");
    assert_eq!(release_in(&headers), Some(version("1.2.3")));
}

#[test]
fn a_repository_with_no_release_yet_has_no_latest_version() {
    let headers = headers_to("https://github.com/daniel-retamal/fishtanks/releases");
    assert_eq!(release_in(&headers), None);
    assert_eq!(release_in(""), None);
}

#[test]
fn a_release_candidate_is_older_than_its_release() {
    assert!(version("1.0.0-rc.1") < version("1.0.0"));
    assert!(version("1.0.0-rc.2") > version("1.0.0-rc.1"));
}

#[test]
fn a_stamp_survives_its_own_file() {
    let stamp = Stamp {
        checked_at: 1_790_000_000,
        latest: Some(version("1.1.0")),
    };
    assert_eq!(Stamp::parse(&stamp.render()), Some(stamp));
    let unanswered = Stamp {
        checked_at: 1_790_000_000,
        latest: None,
    };
    assert_eq!(Stamp::parse(&unanswered.render()), Some(unanswered));
    assert_eq!(Stamp::parse("garbled"), None);
}

#[test]
fn github_is_asked_at_most_once_a_day() {
    let stamp = Stamp {
        checked_at: 1_000,
        latest: None,
    };
    assert!(!stamp.is_due(1_000 + DAY_SECS - 1));
    assert!(stamp.is_due(1_000 + DAY_SECS));
}

#[test]
fn a_newer_version_seen_yesterday_is_announced_without_asking_again() {
    let stamp = Stamp {
        checked_at: 0,
        latest: Some(version("1.1.0")),
    };
    let mut watch = Watch::remembering(version("1.0.0"), Some(&stamp));
    assert!(watch.poll());
    assert_eq!(
        watch.news().as_deref(),
        Some(format!("{NAME} 1.1.0 is out (you have 1.0.0). Run: {NAME} update").as_str())
    );
}

#[test]
fn the_version_already_running_is_never_news() {
    let stamp = Stamp {
        checked_at: 0,
        latest: Some(version("1.0.0")),
    };
    let mut watch = Watch::remembering(version("1.0.0"), Some(&stamp));
    assert!(!watch.poll());
    assert_eq!(watch.news(), None);
    assert!(!Watch::remembering(version("1.0.0"), None).poll());
}

#[test]
fn each_copy_is_told_how_it_updates() {
    let brew = Path::new("/opt/homebrew/Cellar/fishtanks/1.0.0/bin/fishtanks");
    let linuxbrew = Path::new("/home/linuxbrew/.linuxbrew/bin/fishtanks");
    let cargo = Path::new(r"C:\Users\Nemo\.cargo\bin\fishtanks.exe");
    let installed = Path::new("/home/nemo/.local/bin/fishtanks");
    assert_eq!(Route::of(brew, true), Route::Homebrew);
    assert_eq!(Route::of(linuxbrew, false), Route::Homebrew);
    assert_eq!(Route::of(cargo, false), Route::Cargo);
    assert_eq!(Route::of(installed, true), Route::Installer);
    assert_eq!(Route::of(installed, false), Route::Unknown);
    assert_eq!(Route::Installer.advice(), None);
    assert!(Route::Homebrew.advice().unwrap().contains("brew upgrade"));
    assert!(Route::Cargo.advice().unwrap().contains("cargo install"));
    assert!(Route::Unknown.advice().unwrap().contains("releases/latest"));
}

#[test]
fn the_retired_binary_sits_beside_the_new_one() {
    assert_eq!(
        retired(Path::new(r"C:\bin\fishtanks.exe")),
        Path::new(r"C:\bin\fishtanks.old")
    );
    assert_eq!(
        retired(Path::new("/home/nemo/.local/bin/fishtanks")),
        Path::new("/home/nemo/.local/bin/fishtanks.old")
    );
}

#[test]
fn update_is_a_command_and_help_names_it() {
    let argv = |args: &[&str]| {
        std::iter::once(NAME)
            .chain(args.iter().copied())
            .map(String::from)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        Invocation::from_args(argv(&["update"]).into_iter()),
        Invocation::Update
    );
    assert!(Invocation::help().contains("update"));
}

#[test]
fn the_status_bar_says_an_update_is_available_and_zen_hides_it() {
    let mut tui = Tui::as_player(100, 30);
    tui.screen().expect_absent(UPDATE_LABEL);
    tui.app.announce_update();
    tui.tick_n(1);
    tui.screen().expect_find(UPDATE_LABEL);
    tui.run("/zen");
    tui.screen().expect_absent(UPDATE_LABEL);
}
