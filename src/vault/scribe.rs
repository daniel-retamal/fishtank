use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};

use super::crypt::{Burial, Crypt};
use super::write_save;
use crate::app::SaveFile;

pub struct Letter {
    pub save: SaveFile,
    pub burial: Burial,
}

enum Mail {
    Letter(Box<Letter>),
    Flush(Sender<()>),
}

#[derive(Default)]
struct Health {
    failing: AtomicBool,
    last_error: Mutex<Option<String>>,
}

impl Health {
    fn record(&self, outcome: io::Result<()>) {
        let error = outcome.err().map(|error| error.to_string());
        self.failing.store(error.is_some(), Ordering::Relaxed);
        if let Some(error) = error {
            *self
                .last_error
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = Some(error);
        }
    }
}

pub struct Scribe {
    inbox: Option<Sender<Mail>>,
    worker: Option<JoinHandle<()>>,
    health: Arc<Health>,
}

struct Desk {
    path: PathBuf,
    crypt: Crypt,
    health: Arc<Health>,
    broken: Option<io::Error>,
}

impl Desk {
    fn work(mut self, letters: Receiver<Mail>) {
        while let Ok(first) = letters.recv() {
            let mut latest = None;
            let mut waiting = Vec::new();
            for mail in std::iter::once(first).chain(letters.try_iter()) {
                match mail {
                    Mail::Letter(letter) => latest = Some(self.receive(*letter)),
                    Mail::Flush(reply) => waiting.push(reply),
                }
            }
            if let Some(save) = latest {
                let outcome = self.write(save);
                self.health.record(outcome);
            }
            for reply in waiting {
                let _ = reply.send(());
            }
        }
    }

    fn receive(&mut self, letter: Letter) -> SaveFile {
        if let Err(error) = self.crypt.bury(letter.burial) {
            self.broken = Some(error);
        }
        letter.save
    }

    fn write(&mut self, mut save: SaveFile) -> io::Result<()> {
        if let Some(error) = &self.broken {
            return Err(io::Error::new(error.kind(), error.to_string()));
        }
        save.entomb(self.crypt.flush()?);
        write_save(&self.path, &save)?;
        self.crypt.retire();
        Ok(())
    }
}

impl Scribe {
    pub fn writing_to(path: PathBuf, crypt: Crypt) -> Self {
        let (inbox, letters) = mpsc::channel::<Mail>();
        let health = Arc::new(Health::default());
        let desk = Desk {
            path,
            crypt,
            health: Arc::clone(&health),
            broken: None,
        };
        let worker = thread::spawn(move || desk.work(letters));
        Self {
            inbox: Some(inbox),
            worker: Some(worker),
            health,
        }
    }

    pub fn post(&self, letter: Letter) {
        if let Some(inbox) = &self.inbox {
            let _ = inbox.send(Mail::Letter(Box::new(letter)));
        }
    }

    pub fn flush(&self) {
        let Some(inbox) = &self.inbox else {
            return;
        };
        let (reply, done) = mpsc::channel();
        if inbox.send(Mail::Flush(reply)).is_ok() {
            let _ = done.recv();
        }
    }

    pub fn is_failing(&self) -> bool {
        self.health.failing.load(Ordering::Relaxed)
    }

    pub fn last_error(&self) -> Option<String> {
        self.health
            .last_error
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Drop for Scribe {
    fn drop(&mut self) {
        self.inbox.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
