use fishtank::app::Launch;
use fishtank::cli::{Invocation, NAME, VERSION};

fn invoke(args: &[&str]) -> Invocation {
    let argv: Vec<String> = std::iter::once(NAME)
        .chain(args.iter().copied())
        .map(String::from)
        .collect();
    Invocation::from_args(argv.into_iter())
}

#[test]
fn no_arguments_opens_the_players_tank() {
    assert_eq!(invoke(&[]), Invocation::Play(Launch::Player));
}

#[test]
fn the_debug_flag_still_opens_the_lab_bench() {
    assert_eq!(
        invoke(&[Launch::DEBUG_FLAG]),
        Invocation::Play(Launch::Debug)
    );
}

#[test]
fn version_answers_in_both_spellings_without_opening_the_game() {
    assert_eq!(invoke(&["--version"]), Invocation::Version);
    assert_eq!(invoke(&["-V"]), Invocation::Version);
    assert_eq!(Invocation::version(), format!("fishtanks {VERSION}"));
}

#[test]
fn help_wins_over_everything_else_and_names_the_command() {
    assert_eq!(invoke(&["--version", "--help"]), Invocation::Help);
    assert_eq!(invoke(&["-h"]), Invocation::Help);
    assert!(Invocation::help().contains("usage: fishtanks"));
    assert!(Invocation::help().contains(VERSION));
}

#[test]
fn the_program_name_itself_is_never_read_as_a_flag() {
    let args = ["--version".to_string()].into_iter();
    assert_eq!(
        Invocation::from_args(args),
        Invocation::Play(Launch::Player)
    );
}
