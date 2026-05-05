use std::time::Duration;

use anyhow::Result;
use crossterm::event;

mod app;
mod commands;
mod entities;
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

    loop {
        let tick_duration = Duration::from_secs_f32(1.0 / app.settings.fps);

        terminal.draw(|f| app.draw(f))?;

        if event::poll(tick_duration)? {
            app.handle_input(event::read()?);
        } else {
            app.tick();
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
