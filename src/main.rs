use std::{error::Error, time::{Duration, Instant}};
use crossterm::event::{self, Event};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod ui;
mod dotdo;
mod controller;

fn main() -> Result<(), Box<dyn Error>> {
    // Terminal setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new();
    let tick_rate = Duration::from_millis(app.tick_rate_ms);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if controller::handle_input(&mut app, key) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.generate_row();
            last_tick = Instant::now();
        }
    }

    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;

    if app.is_goal_reached() {
        println!("\n\x1b[1;33m--- THE BARD'S TALE ---\x1b[0m");
        println!("The Knight reached the Citadel after a long journey.");
        println!("Monsters slain: \x1b[1;32m{}\x1b[0m", app.slain_count);
        println!("The realm is safer thanks to your focus.\n");
    }

    Ok(())
}