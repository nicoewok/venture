use crossterm::event::{KeyCode, KeyEvent};
use crate::app::{App, Scene};
use std::process::Command;

pub fn handle_input(app: &mut App, key: KeyEvent) -> bool {
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
                KeyCode::Char('q') => return true,
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
                    let max_pos = app.available_tasks.len() + app.custom_monsters.len() + 1;
                    if app.cursor_pos < max_pos {
                        app.cursor_pos += 1;
                    }
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    let tasks_len = app.available_tasks.len();
                    let custom_len = app.custom_monsters.len();
                    
                    if app.cursor_pos < tasks_len {
                        if app.selected_indices.contains(&app.cursor_pos) {
                            app.selected_indices.remove(&app.cursor_pos);
                        } else {
                            app.selected_indices.insert(app.cursor_pos);
                        }
                    } else if app.cursor_pos < tasks_len + custom_len {
                        // Custom monsters toggle?
                    } else if app.cursor_pos == tasks_len + custom_len {
                        app.state = Scene::Naming;
                        app.naming_input.clear();
                    } else if app.cursor_pos == tasks_len + custom_len + 1 {
                        let selected_count = app.selected_indices.len() + app.custom_monsters.len();
                        if selected_count == app.monsters_goal as usize {
                            app.monsters.clear();
                            for &idx in &app.selected_indices {
                                let task = &app.available_tasks[idx];
                                app.monsters.push(crate::app::Monster {
                                    name: task.title.clone(),
                                    is_slain: false,
                                    monster_type: "Bounty".to_string(),
                                    dotdo_id: Some(task.id),
                                    art_idx: idx % 5,
                                });
                            }
                            for (i, name) in app.custom_monsters.iter().enumerate() {
                                app.monsters.push(crate::app::Monster {
                                    name: name.clone(),
                                    is_slain: false,
                                    monster_type: "Wild".to_string(),
                                    dotdo_id: None,
                                    art_idx: (tasks_len + i) % 5,
                                });
                            }
                            app.state = Scene::March;
                        }
                    }
                }
                KeyCode::Char('q') => return true,
                _ => {}
            }
        }
        Scene::Naming => {
            match key.code {
                KeyCode::Enter => {
                    if !app.naming_input.is_empty() {
                        app.custom_monsters.push(app.naming_input.clone());
                        app.state = Scene::Tavern;
                        app.cursor_pos = app.available_tasks.len() + app.custom_monsters.len() - 1;
                    }
                }
                KeyCode::Esc => {
                    app.state = Scene::Tavern;
                }
                KeyCode::Char(c) => {
                    app.naming_input.push(c);
                }
                KeyCode::Backspace => {
                    app.naming_input.pop();
                }
                _ => {}
            }
        }
        Scene::March => {
            match key.code {
                KeyCode::Char('s') => {
                    if !app.monsters.is_empty() {
                        app.state = Scene::Selection;
                        app.cursor_pos = 0;
                    }
                }
                KeyCode::Char('c') => {
                    if app.is_quest_finished() {
                        app.state = Scene::Summary;
                    }
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    app.is_paused = !app.is_paused;
                }
                KeyCode::Char('q') => return true,
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
                    if !app.monsters.is_empty() && app.cursor_pos + 1 < app.monsters.len() {
                        app.cursor_pos += 1;
                    }
                }
                KeyCode::Enter => {
                    if !app.monsters.is_empty() {
                        app.state = Scene::Battle(app.cursor_pos);
                    }
                }
                KeyCode::Char('s') | KeyCode::Esc => {
                    app.state = Scene::March;
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    app.is_paused = !app.is_paused;
                }
                KeyCode::Char('q') => return true,
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
                        app.animation = Some(crate::ui::Animation::Slash { frame: 0 });
                    }
                }
                KeyCode::Char('f') | KeyCode::Esc => {
                    app.state = Scene::March;
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    app.is_paused = !app.is_paused;
                }
                KeyCode::Char('q') => return true,
                _ => {}
            }
        }
        Scene::Summary => {
            match key.code {
                KeyCode::Char('q') | KeyCode::Enter | KeyCode::Esc => return true,
                _ => {}
            }
        }
    }
    false
}
