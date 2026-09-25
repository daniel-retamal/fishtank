use std::process::ExitCode;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{event, terminal};
use fishtank::app::{App, Launch};
use fishtank::closing;
use fishtank::vault::Vault;

const FALLBACK_SIZE: (u16, u16) = (80, 24);

fn main() -> ExitCode {
    let launch = Launch::from_args(std::env::args());
    let vault = match Vault::open(launch) {
        Ok(vault) => vault,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let (width, height) = terminal::size().unwrap_or(FALLBACK_SIZE);
    let mut app = match App::open(launch, vault, width, height) {
        Ok(app) => app,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    closing::listen();
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    let saved = app.persist_and_wait();
    ratatui::restore();
    if let Err(error) = saved {
        eprintln!("fishtank could not write its water down: {error}");
    }
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    let mut last_tick = Instant::now();

    loop {
        let tick_duration = Duration::from_secs_f32(1.0 / app.settings.fps);
        let timeout = tick_duration.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            app.handle_input(event::read()?);
            while event::poll(Duration::ZERO)? {
                app.handle_input(event::read()?);
            }
        }

        if last_tick.elapsed() >= tick_duration {
            app.tick();
            last_tick = Instant::now();
        }

        terminal.draw(|f| app.draw(f))?;
        app.persist_if_due();

        if !app.running || closing::requested() {
            break;
        }
    }

    Ok(())
}
