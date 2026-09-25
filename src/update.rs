use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use semver::Version;

use crate::cli::{NAME, VERSION};

pub const UPDATE_LABEL: &str = "update available";
pub const OPT_OUT_VAR: &str = "FISHTANKS_NO_UPDATE_CHECK";
pub const STAMP_FILE: &str = "update-check";
pub const REPOSITORY: &str = "https://github.com/daniel-retamal/fishtanks";
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const PROBE_TIMEOUT_SECS: &str = "5";
const LOCATION_HEADER: &str = "location:";
const TAG_PREFIX: char = 'v';
const RETIRED_EXTENSION: &str = "old";
const RECEIPT_SUFFIX: &str = "-receipt.json";
const UNIX_CONFIG_DIR: &str = ".config";
const XDG_CONFIG_VAR: &str = "XDG_CONFIG_HOME";
const HOMEBREW_MARKERS: [&str; 3] = ["/cellar/", "/homebrew/", "/linuxbrew/"];
const CARGO_MARKER: &str = "/.cargo/bin/";

pub fn running() -> Version {
    Version::parse(VERSION).unwrap_or_else(|_| Version::new(0, 0, 0))
}

pub fn release_in(headers: &str) -> Option<Version> {
    let location = headers.lines().find_map(|line| {
        line.to_ascii_lowercase()
            .starts_with(LOCATION_HEADER)
            .then(|| line[LOCATION_HEADER.len()..].trim())
    })?;
    let tag = location.trim_end_matches('/').rsplit('/').next()?;
    Version::parse(tag.strip_prefix(TAG_PREFIX).unwrap_or(tag)).ok()
}

fn latest_release() -> Option<Version> {
    let output = Command::new("curl")
        .args(["-sI", "--max-time", PROBE_TIMEOUT_SECS])
        .arg(format!("{REPOSITORY}/releases/latest"))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    release_in(&String::from_utf8_lossy(&output.stdout))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub checked_at: u64,
    pub latest: Option<Version>,
}

impl Stamp {
    pub fn now(latest: Option<Version>) -> Self {
        Self {
            checked_at: seconds_since_epoch(SystemTime::now()),
            latest,
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        let mut words = text.split_whitespace();
        let checked_at = words.next()?.parse().ok()?;
        let latest = words.next().and_then(|word| Version::parse(word).ok());
        Some(Self { checked_at, latest })
    }

    pub fn render(&self) -> String {
        match &self.latest {
            Some(latest) => format!("{} {latest}", self.checked_at),
            None => self.checked_at.to_string(),
        }
    }

    pub fn is_due(&self, now: u64) -> bool {
        now.saturating_sub(self.checked_at) >= CHECK_INTERVAL.as_secs()
    }

    fn read(path: &Path) -> Option<Self> {
        Self::parse(&fs::read_to_string(path).ok()?)
    }
}

fn seconds_since_epoch(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

pub struct Watch {
    running: Version,
    found: Option<Version>,
    pending: Option<Receiver<Option<Version>>>,
}

impl Watch {
    pub fn start(save: &Path) -> Self {
        let running = running();
        if env::var_os(OPT_OUT_VAR).is_some() {
            return Self::remembering(running, None);
        }
        let path = save.with_file_name(STAMP_FILE);
        let stamp = Stamp::read(&path);
        let mut watch = Self::remembering(running, stamp.as_ref());
        let due = stamp.is_none_or(|stamp| stamp.is_due(seconds_since_epoch(SystemTime::now())));
        if due {
            watch.pending = Some(Self::ask_in_the_background(path));
        }
        watch
    }

    pub fn remembering(running: Version, stamp: Option<&Stamp>) -> Self {
        let found = stamp
            .and_then(|stamp| stamp.latest.clone())
            .filter(|latest| *latest > running);
        Self {
            running,
            found,
            pending: None,
        }
    }

    fn ask_in_the_background(path: PathBuf) -> Receiver<Option<Version>> {
        let (answer, answered) = mpsc::channel();
        thread::spawn(move || {
            let latest = latest_release();
            if latest.is_some() {
                let _ = fs::write(&path, Stamp::now(latest.clone()).render());
            }
            let _ = answer.send(latest);
        });
        answered
    }

    pub fn poll(&mut self) -> bool {
        if let Some(pending) = &self.pending
            && let Ok(latest) = pending.try_recv()
        {
            self.pending = None;
            self.found = latest.filter(|latest| *latest > self.running);
        }
        self.found.is_some()
    }

    pub fn news(&self) -> Option<String> {
        let latest = self.found.as_ref()?;
        Some(format!(
            "{NAME} {latest} is out (you have {}). Run: {NAME} update",
            self.running
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Installer,
    Homebrew,
    Cargo,
    Unknown,
}

impl Route {
    pub fn of(exe: &Path, has_receipt: bool) -> Self {
        let path = exe
            .to_string_lossy()
            .replace('\\', "/")
            .to_ascii_lowercase();
        if HOMEBREW_MARKERS.iter().any(|marker| path.contains(marker)) {
            return Route::Homebrew;
        }
        if path.contains(CARGO_MARKER) {
            return Route::Cargo;
        }
        if has_receipt {
            return Route::Installer;
        }
        Route::Unknown
    }

    fn detect() -> Self {
        let Ok(exe) = env::current_exe() else {
            return Route::Unknown;
        };
        let has_receipt = receipt_path().is_some_and(|path| path.exists());
        Self::of(&exe, has_receipt)
    }

    pub fn advice(self) -> Option<String> {
        match self {
            Route::Installer => None,
            Route::Homebrew => Some(format!(
                "This {NAME} came from Homebrew. Update it with: brew upgrade {NAME}"
            )),
            Route::Cargo => Some(format!(
                "This {NAME} was built by cargo. Update it with: cargo install {NAME}"
            )),
            Route::Unknown => Some(format!(
                "This {NAME} was not put here by its installer, so it cannot update itself. The newest one is at {REPOSITORY}/releases/latest"
            )),
        }
    }
}

fn receipt_path() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        dirs::data_local_dir()?
    } else {
        env::var_os(XDG_CONFIG_VAR)
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(UNIX_CONFIG_DIR)))?
    };
    Some(base.join(NAME).join(format!("{NAME}{RECEIPT_SUFFIX}")))
}

pub fn retired(exe: &Path) -> PathBuf {
    exe.with_extension(RETIRED_EXTENSION)
}

pub fn sweep() {
    if let Ok(exe) = env::current_exe() {
        let _ = fs::remove_file(retired(&exe));
    }
}

fn installer() -> Command {
    let download = format!("{REPOSITORY}/releases/latest/download/{NAME}-installer");
    if cfg!(windows) {
        let mut command = Command::new("powershell");
        command
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"])
            .arg(format!("irm {download}.ps1 | iex"));
        return command;
    }
    let mut command = Command::new("sh");
    command.arg("-c").arg(format!(
        "curl --proto '=https' --tlsv1.2 -LsSf {download}.sh | sh"
    ));
    command
}

pub fn run() -> ExitCode {
    let route = Route::detect();
    if let Some(advice) = route.advice() {
        println!("{advice}");
        return ExitCode::SUCCESS;
    }
    let running = running();
    let Some(latest) = latest_release() else {
        eprintln!("{NAME} could not reach GitHub to look for a newer version.");
        return ExitCode::FAILURE;
    };
    if latest <= running {
        println!("{NAME} {running} is the newest there is.");
        return ExitCode::SUCCESS;
    }
    println!("Updating {NAME} {running} to {latest}.");
    reinstall()
}

fn reinstall() -> ExitCode {
    let Ok(exe) = env::current_exe() else {
        eprintln!("{NAME} could not find itself on disk.");
        return ExitCode::FAILURE;
    };
    let aside = retired(&exe);
    if fs::rename(&exe, &aside).is_err() {
        eprintln!("{NAME} could not move itself aside to make room for the new version.");
        return ExitCode::FAILURE;
    }
    let installed = installer().status().is_ok_and(|status| status.success());
    if !installed || !exe.exists() {
        let _ = fs::rename(&aside, &exe);
        eprintln!("The installer did not finish, so {NAME} was left as it was.");
        return ExitCode::FAILURE;
    }
    let _ = fs::remove_file(&aside);
    ExitCode::SUCCESS
}
