use std::fs;
use std::path::{Path, PathBuf};

use super::{Screenplay, Tui};

pub const PLAY_EXTENSION: &str = "play";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

impl TerminalSize {
    pub const fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }

    pub fn label(self) -> String {
        format!("{}x{}", self.cols, self.rows)
    }
}

pub fn plays_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut plays: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == PLAY_EXTENSION))
        .collect();
    plays.sort();
    plays
}

pub fn play_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_string()
}

pub fn stage(
    path: &Path,
    size: TerminalSize,
    reel_dir: &Path,
    reel_name: &str,
) -> Result<Tui, String> {
    let play = Screenplay::load(path).map_err(|error| format!("{reel_name}: {error}"))?;
    let mut tui = Tui::with_size(size.cols, size.rows);
    tui.film(reel_dir, reel_name);
    play.perform(&mut tui)
        .map_err(|error| format!("{reel_name}: {error}"))?;
    Ok(tui)
}

pub fn review(performance: Result<Tui, String>, reel_name: &str) -> Option<String> {
    match performance {
        Err(failure) => Some(failure),
        Ok(tui) => {
            let report = tui.reel().flaw_report();
            (!report.is_empty()).then(|| format!("{reel_name}:\n{report}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_names_itself_the_way_a_reel_file_is_named() {
        assert_eq!(TerminalSize::new(60, 18).label(), "60x18");
    }

    #[test]
    fn a_missing_directory_holds_no_plays() {
        assert!(plays_in(Path::new("no/such/stage")).is_empty());
    }

    #[test]
    fn a_play_is_named_after_its_file() {
        assert_eq!(
            play_name(Path::new("tests/journeys/sell-a-fish.play")),
            "sell-a-fish"
        );
    }
}
