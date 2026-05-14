use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event;

mod app;
mod commands;
mod consumable;
mod entities;
mod loot;
mod settings;
mod tank;
mod ui;

use app::App;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let mut app = App::new();
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

        if !app.running {
            break;
        }
    }

    Ok(())
}
