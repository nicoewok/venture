use std::{error::Error, time::{Duration, Instant}};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod ui;
mod dotdo;

use std::process::Command;

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
                use app::Scene;
                match app.state {
                    Scene::Setup => {
                        match key.code {
                            KeyCode::Up => {
                                if app.setup_cursor > 0 {
                                    app.setup_cursor -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.setup_cursor < 2 {
                                    app.setup_cursor += 1;
                                }
                            }
                            KeyCode::Left => {
                                match app.setup_cursor {
                                    0 => if app.hours > 0 { app.hours -= 1 },
                                    1 => if app.mins >= 5 { app.mins -= 5 } else if app.hours > 0 { app.hours -= 1; app.mins = 55 },
                                    2 => if app.monsters_goal > 1 { app.monsters_goal -= 1 },
                                    _ => {}
                                }
                            }
                            KeyCode::Right => {
                                match app.setup_cursor {
                                    0 => app.hours += 1,
                                    1 => if app.mins < 55 { app.mins += 5 } else { app.hours += 1; app.mins = 0 },
                                    2 => app.monsters_goal += 1,
                                    _ => {}
                                }
                            }
                            KeyCode::Enter => {
                                app.state = Scene::Tavern;
                                app.cursor_pos = 0;
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                    Scene::Tavern => {
                        match key.code {
                            KeyCode::Up => {
                                if app.cursor_pos > 0 {
                                    app.cursor_pos -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.cursor_pos < app.available_tasks.len() {
                                    app.cursor_pos += 1;
                                }
                            }
                            KeyCode::Char(' ') => {
                                if app.cursor_pos < app.available_tasks.len() {
                                    if app.selected_indices.contains(&app.cursor_pos) {
                                        app.selected_indices.remove(&app.cursor_pos);
                                    } else {
                                        app.selected_indices.insert(app.cursor_pos);
                                    }
                                } else {
                                    // Clicked [+]
                                    app.custom_monster_count += 1;
                                }
                            }
                            KeyCode::Enter => {
                                // Sync selected tasks to monsters
                                app.monsters.clear();
                                for &idx in &app.selected_indices {
                                    let task = &app.available_tasks[idx];
                                    app.monsters.push(app::Monster {
                                        name: task.title.clone(),
                                        is_slain: false,
                                        monster_type: "Bounty".to_string(),
                                        dotdo_id: Some(task.id),
                                    });
                                }
                                // Add custom monsters
                                for i in 1..=app.custom_monster_count {
                                    app.monsters.push(app::Monster {
                                        name: format!("Custom Monster #{}", i),
                                        is_slain: false,
                                        monster_type: "Wild".to_string(),
                                        dotdo_id: None,
                                    });
                                }
                                app.state = Scene::March;
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                    Scene::March => {
                        match key.code {
                            KeyCode::Char('s') => {
                                app.state = Scene::Selection;
                                app.cursor_pos = 0;
                            }
                            KeyCode::Char('c') => {
                                if app.is_goal_reached() {
                                    break;
                                }
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                    Scene::Selection => {
                        match key.code {
                            KeyCode::Up => {
                                if app.cursor_pos > 0 {
                                    app.cursor_pos -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.cursor_pos + 1 < app.monsters.len() {
                                    app.cursor_pos += 1;
                                }
                            }
                            KeyCode::Enter => {
                                app.state = Scene::Battle(app.cursor_pos);
                            }
                            KeyCode::Char('s') | KeyCode::Esc => {
                                app.state = Scene::March;
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                    Scene::Battle(idx) => {
                        match key.code {
                            KeyCode::Char('s') => {
                                if !app.monsters[idx].is_slain {
                                    app.monsters[idx].is_slain = true;
                                    app.slain_count += 1;
                                    if let Some(id) = app.monsters[idx].dotdo_id {
                                        let _ = Command::new("dotdo")
                                            .arg("done")
                                            .arg(id.to_string())
                                            .spawn();
                                    }
                                }
                            }
                            KeyCode::Char('f') | KeyCode::Esc => {
                                app.state = Scene::March;
                            }
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.generate_row(); // This makes the path "move"
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