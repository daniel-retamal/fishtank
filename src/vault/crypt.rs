use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::thread;

use rand::RngExt;
use serde::{Deserialize, Serialize};

use super::seal::Seal;
use super::write_atomically;
use crate::fishes::fish::Fish;

const CRYPT_EXTENSION: &str = "crypt";
const LINE_BREAK: u8 = b'\n';
const ID_DIGITS: usize = 16;
const ID_RADIX: u32 = 16;
const GRAVES_PER_WORKER_MIN: usize = 512;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CryptSeal {
    pub id: u64,
    pub graves: usize,
    pub seal: Seal,
}

pub struct Burial {
    pub keep: usize,
    pub fresh: Vec<Fish>,
}

pub struct Unearthed {
    pub dead: Vec<Fish>,
    pub crypt: Crypt,
    pub intact: bool,
}

pub struct Crypt {
    save_path: PathBuf,
    id: u64,
    ends: Vec<u64>,
    seal: Seal,
    keep: Option<usize>,
    staged: Vec<String>,
    retired: Vec<u64>,
}

impl Crypt {
    pub fn empty(save_path: &Path) -> Self {
        Self {
            save_path: save_path.to_path_buf(),
            id: rand::rng().random(),
            ends: Vec::new(),
            seal: Seal::default(),
            keep: None,
            staged: Vec::new(),
            retired: Vec::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn holds(&self) -> usize {
        self.keep.unwrap_or(self.ends.len())
    }

    pub fn path_for(save_path: &Path, id: u64) -> PathBuf {
        let mut name = stem_of(save_path);
        name.push(format!(
            ".{id:0width$x}.{CRYPT_EXTENSION}",
            width = ID_DIGITS
        ));
        save_path.with_file_name(name)
    }

    pub fn unearth(save_path: &Path, sealed: &CryptSeal) -> io::Result<Unearthed> {
        let mut crypt = Self::empty(save_path);
        crypt.id = sealed.id;
        let bytes = match fs::read(Self::path_for(save_path, sealed.id)) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error),
        };
        let (dead, readable) = crypt.read_graves(&bytes, sealed.graves);
        let intact = readable && dead.len() == sealed.graves && crypt.seal == sealed.seal;
        if !intact {
            crypt.keep = Some(0);
        }
        Ok(Unearthed {
            dead,
            crypt,
            intact,
        })
    }

    fn read_graves(&mut self, bytes: &[u8], graves: usize) -> (Vec<Fish>, bool) {
        let mut lines = Vec::with_capacity(graves);
        let mut start = 0usize;
        for line in bytes
            .split_inclusive(|&byte| byte == LINE_BREAK)
            .take(graves)
        {
            if line.last() != Some(&LINE_BREAK) {
                break;
            }
            start += line.len();
            self.seal.feed(line);
            self.ends.push(start as u64);
            lines.push(&line[..line.len() - 1]);
        }
        let raised = raise(&lines);
        let readable = raised.len() == lines.len() && raised.iter().all(Option::is_some);
        (raised.into_iter().flatten().collect(), readable)
    }

    pub fn bury(&mut self, burial: Burial) -> io::Result<()> {
        let kept = self.keep.unwrap_or(self.ends.len());
        if burial.keep < kept {
            self.keep = Some(burial.keep);
            self.staged.clear();
        } else {
            self.staged.truncate(burial.keep - kept);
        }
        for fish in &burial.fresh {
            self.staged
                .push(ron::to_string(fish).map_err(io::Error::other)?);
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<CryptSeal> {
        match self.keep {
            Some(kept) => self.rewrite(kept)?,
            None if !self.staged.is_empty() => self.append()?,
            None => {}
        }
        Ok(CryptSeal {
            id: self.id,
            graves: self.ends.len(),
            seal: self.seal,
        })
    }

    fn staged_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.staged.iter().map(|line| line.len() + 1).sum());
        for line in &self.staged {
            bytes.extend_from_slice(line.as_bytes());
            bytes.push(LINE_BREAK);
        }
        bytes
    }

    fn settle_staged(&mut self, from: u64) {
        let mut end = from;
        for line in std::mem::take(&mut self.staged) {
            end += line.len() as u64 + 1;
            self.ends.push(end);
        }
    }

    fn append(&mut self) -> io::Result<()> {
        let valid = self.ends.last().copied().unwrap_or_default();
        let staged = self.staged_bytes();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(Self::path_for(&self.save_path, self.id))?;
        file.set_len(valid)?;
        file.seek(SeekFrom::Start(valid))?;
        file.write_all(&staged)?;
        file.sync_all()?;
        self.seal.feed(&staged);
        self.settle_staged(valid);
        Ok(())
    }

    fn rewrite(&mut self, kept: usize) -> io::Result<()> {
        let prefix_end = kept.checked_sub(1).map_or(0, |last| self.ends[last]);
        let mut bytes = Vec::new();
        if prefix_end > 0 {
            File::open(Self::path_for(&self.save_path, self.id))?
                .take(prefix_end)
                .read_to_end(&mut bytes)?;
        }
        if bytes.len() as u64 != prefix_end {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        bytes.extend(self.staged_bytes());
        let id = self.fresh_id();
        write_atomically(&Self::path_for(&self.save_path, id), &bytes)?;
        self.retired.push(self.id);
        self.id = id;
        self.ends.truncate(kept);
        self.seal = Seal::of(&bytes[..prefix_end as usize]);
        self.seal.feed(&bytes[prefix_end as usize..]);
        self.settle_staged(prefix_end);
        self.keep = None;
        Ok(())
    }

    fn fresh_id(&self) -> u64 {
        let mut rng = rand::rng();
        loop {
            let id: u64 = rng.random();
            if id != self.id && !self.retired.contains(&id) {
                return id;
            }
        }
    }

    pub fn retire(&mut self) {
        for id in self.retired.drain(..) {
            let _ = fs::remove_file(Self::path_for(&self.save_path, id));
        }
    }

    pub fn sweep(save_path: &Path, keep: u64) {
        let Some(dir) = save_path.parent() else {
            return;
        };
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        let stem = stem_of(save_path);
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(id) = Self::id_of(&stem, &path)
                && id != keep
            {
                let _ = fs::remove_file(path);
            }
        }
    }

    fn id_of(stem: &OsString, path: &Path) -> Option<u64> {
        if path.extension()? != CRYPT_EXTENSION {
            return None;
        }
        let name = path.file_stem()?.to_str()?;
        let hex = name.strip_prefix(stem.to_str()?)?.strip_prefix('.')?;
        (hex.len() == ID_DIGITS)
            .then(|| u64::from_str_radix(hex, ID_RADIX).ok())
            .flatten()
    }
}

fn raise(lines: &[&[u8]]) -> Vec<Option<Fish>> {
    let workers = thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let share = lines.len().div_ceil(workers).max(GRAVES_PER_WORKER_MIN);
    thread::scope(|scope| {
        let diggers: Vec<_> = lines
            .chunks(share)
            .map(|share| {
                scope.spawn(move || share.iter().map(|line| exhume(line)).collect::<Vec<_>>())
            })
            .collect();
        diggers
            .into_iter()
            .flat_map(|digger| digger.join().unwrap_or_default())
            .collect()
    })
}

fn exhume(line: &[u8]) -> Option<Fish> {
    let text = std::str::from_utf8(line).ok()?;
    ron::from_str::<Fish>(text).ok()
}

fn stem_of(save_path: &Path) -> OsString {
    save_path
        .file_stem()
        .map(OsString::from)
        .unwrap_or_default()
}
