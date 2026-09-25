use std::fmt;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::app::{Launch, SAVE_VERSION, SaveFile};

mod crypt;
mod scribe;
mod seal;

pub use crypt::{Burial, Crypt, CryptSeal};
pub use scribe::{Letter, Scribe};
pub use seal::Seal;

const APP_DIR: &str = "fishtank";
const PLAYER_SLOT: &str = "fishtank";
const DEBUG_SLOT: &str = "debug";
const SAVE_EXTENSION: &str = "ron";
const LOCK_EXTENSION: &str = "lock";
const SCRATCH_SUFFIX: &str = "tmp";
const UNREADABLE_LABEL: &str = "unreadable";

#[derive(Debug)]
pub enum VaultError {
    NoHome,
    AlreadyRunning,
    Io(io::Error),
    Unreachable { path: PathBuf, error: io::Error },
    FromTheFuture { path: PathBuf, version: u32 },
    Stuck { path: PathBuf, error: io::Error },
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VaultError::NoHome => write!(f, "fishtank found no place to keep its water"),
            VaultError::AlreadyRunning => {
                write!(f, "fishtank is already swimming in another window")
            }
            VaultError::Io(error) => write!(f, "fishtank could not reach its water: {error}"),
            VaultError::Unreachable { path, error } => write!(
                f,
                "fishtank could not read its water at {}: {error}\nNothing was touched. Try again once the file can be read.",
                path.display()
            ),
            VaultError::FromTheFuture { path, version } => write!(
                f,
                "the water at {} was poured by a newer fishtank (save version {version}; this one reads up to {SAVE_VERSION})\nNothing was touched. Update fishtank to keep swimming.",
                path.display()
            ),
            VaultError::Stuck { path, error } => write!(
                f,
                "fishtank could not read its water at {} and could not move it aside: {error}\nNothing was touched.",
                path.display()
            ),
        }
    }
}

impl From<io::Error> for VaultError {
    fn from(error: io::Error) -> Self {
        VaultError::Io(error)
    }
}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Garbled(String),
    FromTheFuture(u32),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Io(error) => write!(f, "{error}"),
            ReadError::Garbled(error) => write!(f, "{error}"),
            ReadError::FromTheFuture(version) => {
                write!(f, "save version {version} is newer than {SAVE_VERSION}")
            }
        }
    }
}

pub struct Opened {
    pub save: SaveFile,
    pub intact: bool,
    pub crypt: Crypt,
}

pub enum Loaded {
    Fresh,
    Resumed(Box<Opened>),
    Unreadable { set_aside: PathBuf },
}

#[derive(Deserialize)]
struct Header {
    version: u32,
}

impl Header {
    fn peek(text: &str) -> Result<u32, ReadError> {
        ron::Options::default()
            .without_recursion_limit()
            .from_str::<Header>(text)
            .map(|header| header.version)
            .map_err(|error| ReadError::Garbled(error.to_string()))
    }
}

pub struct Vault {
    dir: PathBuf,
    slot: &'static str,
    _lock: File,
}

impl Vault {
    pub fn open(launch: Launch) -> Result<Self, VaultError> {
        let home = dirs::data_dir().ok_or(VaultError::NoHome)?;
        Self::open_in(home.join(APP_DIR), launch)
    }

    pub fn open_in(dir: PathBuf, launch: Launch) -> Result<Self, VaultError> {
        fs::create_dir_all(&dir)?;
        let slot = match launch {
            Launch::Player => PLAYER_SLOT,
            Launch::Debug => DEBUG_SLOT,
        };
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.join(format!("{slot}.{LOCK_EXTENSION}")))?;
        match lock.try_lock() {
            Ok(()) => Ok(Self {
                dir,
                slot,
                _lock: lock,
            }),
            Err(TryLockError::WouldBlock) => Err(VaultError::AlreadyRunning),
            Err(TryLockError::Error(error)) => Err(VaultError::Io(error)),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join(format!("{}.{SAVE_EXTENSION}", self.slot))
    }

    pub fn load(&self) -> Result<Loaded, VaultError> {
        let path = self.path();
        if !path.exists() {
            return Ok(Loaded::Fresh);
        }
        match read_save(&path) {
            Ok(opened) => {
                Crypt::sweep(&path, opened.crypt.id());
                Ok(Loaded::Resumed(Box::new(opened)))
            }
            Err(ReadError::Io(error)) => Err(VaultError::Unreachable { path, error }),
            Err(ReadError::FromTheFuture(version)) => {
                Err(VaultError::FromTheFuture { path, version })
            }
            Err(ReadError::Garbled(_)) => {
                let set_aside = self.labelled(&format!("{UNREADABLE_LABEL}-{}", unix_secs()));
                fs::rename(&path, &set_aside).map_err(|error| VaultError::Stuck { path, error })?;
                Ok(Loaded::Unreadable { set_aside })
            }
        }
    }

    pub fn scribe(&self, crypt: Crypt) -> Scribe {
        Scribe::writing_to(self.path(), crypt)
    }

    pub fn set_aside(&self, label: &str, save: &SaveFile) -> io::Result<PathBuf> {
        let path = self.labelled(label);
        write_save(&path, save)?;
        Ok(path)
    }

    fn labelled(&self, label: &str) -> PathBuf {
        self.dir
            .join(format!("{}.{label}.{SAVE_EXTENSION}", self.slot))
    }
}

pub fn read_save(path: &Path) -> Result<Opened, ReadError> {
    let text = fs::read_to_string(path).map_err(ReadError::Io)?;
    let mut save = match SaveFile::from_ron(&text) {
        Ok(save) => save,
        Err(error) => {
            let version = Header::peek(&text)?;
            if version > SAVE_VERSION {
                return Err(ReadError::FromTheFuture(version));
            }
            return Err(ReadError::Garbled(error.to_string()));
        }
    };
    if save.version() > SAVE_VERSION {
        return Err(ReadError::FromTheFuture(save.version()));
    }
    let mut intact = Seal::is_intact(&text);
    let crypt = match save.crypt() {
        Some(sealed) => {
            let unearthed = Crypt::unearth(path, &sealed).map_err(ReadError::Io)?;
            intact &= unearthed.intact;
            save.lay_to_rest(unearthed.dead);
            unearthed.crypt
        }
        None => Crypt::empty(path),
    };
    Ok(Opened {
        save,
        intact,
        crypt,
    })
}

pub fn write_save(path: &Path, save: &SaveFile) -> io::Result<()> {
    let body = save.to_ron().map_err(io::Error::other)?;
    write_atomically(path, Seal::stamp(&body).as_bytes())
}

fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    let mut scratch_name = path.as_os_str().to_owned();
    scratch_name.push(format!(".{SCRATCH_SUFFIX}"));
    let scratch = PathBuf::from(scratch_name);
    {
        let mut file = File::create(&scratch)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(&scratch, path)
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}
