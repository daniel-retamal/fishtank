use crate::app::Launch;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const TAGLINE: &str = "An aquarium for your terminal.";
const HELP_FLAGS: [&str; 2] = ["--help", "-h"];
const VERSION_FLAGS: [&str; 2] = ["--version", "-V"];
const UPDATE_COMMAND: &str = "update";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Invocation {
    Play(Launch),
    Help,
    Version,
    Update,
}

impl Invocation {
    pub fn from_args(args: impl Iterator<Item = String>) -> Self {
        let args: Vec<String> = args.skip(1).collect();
        let given = |flags: [&str; 2]| args.iter().any(|arg| flags.contains(&arg.as_str()));
        if given(HELP_FLAGS) {
            return Invocation::Help;
        }
        if given(VERSION_FLAGS) {
            return Invocation::Version;
        }
        if args.first().is_some_and(|first| first == UPDATE_COMMAND) {
            return Invocation::Update;
        }
        Invocation::Play(Launch::from_args(args.into_iter()))
    }

    pub fn version() -> String {
        format!("{NAME} {VERSION}")
    }

    pub fn help() -> String {
        format!(
            "{version}\n{TAGLINE}\n\nusage: {NAME} [{UPDATE_COMMAND}]\n\nRun it with no arguments to open your tank, and type /exit to leave.\n\n  {UPDATE_COMMAND}         install the newest version\n  -h, --help     print this\n  -V, --version  print the version",
            version = Self::version()
        )
    }
}
