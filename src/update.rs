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
const HOMEBREW_MARKERS: [&str; 3] = ["/cellar/", "/homebrew/", "/linuxbrew/"];
const CARGO_MARKER: &str = "/.cargo/bin/";
const SHIPPED_ARCHES: [&str; 2] = ["x86_64", "aarch64"];
const WINDOWS_ARCHIVE: &str = "zip";
const UNIX_ARCHIVE: &str = "tar.xz";
const DOWNLOAD_TIMEOUT_SECS: &str = "120";
const SYSTEM_ROOT_VAR: &str = "SystemRoot";
const WINDOWS_ROOT: &str = r"C:\Windows";
const WINDOWS_TOOLS: &str = "System32";
const WINDOWS_TAR: &str = "tar.exe";

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

fn ask_github() -> Option<String> {
    let output = Command::new("curl")
        .args(["-sI", "--max-time", PROBE_TIMEOUT_SECS])
        .arg(format!("{REPOSITORY}/releases/latest"))
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let headers = String::from_utf8_lossy(&output.stdout).into_owned();
    (!headers.trim().is_empty()).then_some(headers)
}

fn latest_release() -> Option<Version> {
    release_in(&ask_github()?)
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
    Itself,
    Homebrew,
    Cargo,
}

impl Route {
    pub fn of(exe: &Path) -> Self {
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
        Route::Itself
    }

    pub fn advice(self) -> Option<String> {
        match self {
            Route::Itself => None,
            Route::Homebrew => Some(format!(
                "This {NAME} came from Homebrew. Update it with: brew update && brew upgrade {NAME}"
            )),
            Route::Cargo => Some(format!(
                "This {NAME} was built by cargo. Update it with: cargo install {NAME}"
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Archive {
    target: String,
    windows: bool,
}

impl Archive {
    pub fn for_system(os: &str, arch: &str) -> Option<Self> {
        if !SHIPPED_ARCHES.contains(&arch) {
            return None;
        }
        let platform = match os {
            "windows" => "pc-windows-msvc",
            "macos" => "apple-darwin",
            "linux" => "unknown-linux-musl",
            _ => return None,
        };
        Some(Self {
            target: format!("{arch}-{platform}"),
            windows: os == "windows",
        })
    }

    fn this_system() -> Option<Self> {
        Self::for_system(env::consts::OS, env::consts::ARCH)
    }

    pub fn file_name(&self) -> String {
        let extension = if self.windows {
            WINDOWS_ARCHIVE
        } else {
            UNIX_ARCHIVE
        };
        format!("{NAME}-{}.{extension}", self.target)
    }

    pub fn url(&self, version: &Version) -> String {
        format!(
            "{REPOSITORY}/releases/download/{TAG_PREFIX}{version}/{}",
            self.file_name()
        )
    }

    pub fn binary_inside(&self) -> PathBuf {
        if self.windows {
            return PathBuf::from(format!("{NAME}.exe"));
        }
        Path::new(&format!("{NAME}-{}", self.target)).join(NAME)
    }

    fn tar(&self) -> Command {
        if !self.windows {
            return Command::new("tar");
        }
        let root =
            env::var_os(SYSTEM_ROOT_VAR).map_or_else(|| PathBuf::from(WINDOWS_ROOT), PathBuf::from);
        Command::new(root.join(WINDOWS_TOOLS).join(WINDOWS_TAR))
    }

    fn fetch_into(&self, version: &Version, dir: &Path) -> Option<PathBuf> {
        let archive = dir.join(self.file_name());
        let downloaded = Command::new("curl")
            .args(["-fsSL", "--max-time", DOWNLOAD_TIMEOUT_SECS, "-o"])
            .arg(&archive)
            .arg(self.url(version))
            .stdin(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if !downloaded {
            return None;
        }
        let unpacked = self
            .tar()
            .arg("-xf")
            .arg(&archive)
            .arg("-C")
            .arg(dir)
            .stdin(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        let binary = dir.join(self.binary_inside());
        (unpacked && binary.exists()).then_some(binary)
    }
}

pub fn retired(exe: &Path) -> PathBuf {
    exe.with_extension(RETIRED_EXTENSION)
}

pub fn sweep() {
    if let Ok(exe) = env::current_exe() {
        let _ = fs::remove_file(retired(&exe));
    }
}

pub fn run() -> ExitCode {
    let Ok(exe) = env::current_exe() else {
        eprintln!("{NAME} could not find itself on disk.");
        return ExitCode::FAILURE;
    };
    if let Some(advice) = Route::of(&exe).advice() {
        println!("{advice}");
        return ExitCode::SUCCESS;
    }
    let running = running();
    let Some(headers) = ask_github() else {
        eprintln!("{NAME} could not reach GitHub to look for a newer version.");
        return ExitCode::FAILURE;
    };
    let Some(latest) = release_in(&headers) else {
        println!("No {NAME} release has been published yet, so {running} is the newest there is.");
        return ExitCode::SUCCESS;
    };
    if latest <= running {
        println!("{NAME} {running} is the newest there is.");
        return ExitCode::SUCCESS;
    }
    let Some(archive) = Archive::this_system() else {
        println!(
            "There is no {NAME} build for this system. The newest one is at {REPOSITORY}/releases/latest"
        );
        return ExitCode::FAILURE;
    };
    println!("Updating {NAME} {running} to {latest}.");
    let workshop = env::temp_dir().join(format!("{NAME}-{latest}"));
    let _ = fs::remove_dir_all(&workshop);
    let outcome = fs::create_dir_all(&workshop)
        .ok()
        .and_then(|_| archive.fetch_into(&latest, &workshop))
        .map(|fresh| swap(&exe, &fresh));
    let _ = fs::remove_dir_all(&workshop);
    match outcome {
        Some(true) => {
            println!("{NAME} is now {latest}.");
            ExitCode::SUCCESS
        }
        Some(false) => {
            eprintln!("{NAME} could not replace itself, so it was left as it was.");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("{NAME} could not download {latest}, so it was left as it was.");
            ExitCode::FAILURE
        }
    }
}

fn swap(exe: &Path, fresh: &Path) -> bool {
    let aside = retired(exe);
    let _ = fs::remove_file(&aside);
    if fs::rename(exe, &aside).is_err() {
        return false;
    }
    if fs::copy(fresh, exe).is_err() {
        let _ = fs::rename(&aside, exe);
        return false;
    }
    let _ = fs::remove_file(&aside);
    true
}
