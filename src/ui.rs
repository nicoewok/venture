use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub mod map;
pub mod monster;
pub mod tavern;
pub mod castle;

pub use map::{WorldMap, CellType};

pub enum Animation {
    Slash { frame: usize },
}

use crate::app::{App, Scene};

const KNIGHT: &str = "K";
const MONSTER: &str = "M";
const SLAIN: &str = "X";

pub fn render(f: &mut Frame, app: &mut App) {
    match app.state {
        Scene::Setup => draw_setup(f, app),
        Scene::Tavern => tavern::draw_tavern(f, app),
        Scene::Naming => draw_naming(f, app),
        Scene::March => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(f.area());

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(3)])
                .split(main_chunks[0]);

            render_march(f, app, left_chunks[0]);
            render_footer(f, app, left_chunks[1]);

            if main_chunks[1].width > 0 {
                render_sidebar(f, app, main_chunks[1]);
            }
        }
        Scene::Battle(idx) => monster::render_battle(f, app, idx),
        Scene::Selection => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(f.area());

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(3)])
                .split(main_chunks[0]);

            render_march(f, app, left_chunks[0]);
            render_footer(f, app, left_chunks[1]);

            if main_chunks[1].width > 0 {
                render_sidebar(f, app, main_chunks[1]);
            }
            
            draw_selection(f, app);
        }
    }

    if app.is_paused {
        draw_paused_overlay(f);
    }
}

pub fn draw_setup(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(3),    // Inputs
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    let title = Paragraph::new("--- QUEST PREPARATION ---")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let hours_style = if app.setup_cursor == 0 { Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD) } else { Style::default() };
    let mins_style = if app.setup_cursor == 1 { Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD) } else { Style::default() };
    let monsters_style = if app.setup_cursor == 2 { Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD) } else { Style::default() };

    let setup_text = vec![
        Line::from(vec![
            Span::styled("    Hours: ", Style::default()),
            Span::styled(format!("{:02}", app.hours), hours_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Minutes: ", Style::default()),
            Span::styled(format!("{:02}", app.mins), mins_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Monsters Goal: ", Style::default()),
            Span::styled(format!("{}", app.monsters_goal), monsters_style),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from("    (Use Up/Down to switch fields, Left/Right to adjust)"),
    ];
    
    let setup_panel = Paragraph::new(setup_text)
        .block(Block::default().borders(Borders::ALL).title(" Initial Settings "));
    f.render_widget(setup_panel, chunks[1]);

    let footer = Paragraph::new("[Enter] Confirm & Enter Tavern | [Q] Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}


pub fn draw_naming(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Length(3), Constraint::Percentage(40)])
        .split(area);
    let target = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)])
        .split(vertical[1])[1];

    let input = Paragraph::new(app.naming_input.clone())
        .block(Block::default().borders(Borders::ALL).title(" Enter Monster Name "));
    f.render_widget(Clear, target);
    f.render_widget(input, target);
}

pub fn draw_selection(f: &mut Frame, app: &mut App) {
    let area = f.area();
    
    // Create a centered rect for the overlay
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);
    
    let target_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1])[1];

    let monsters: Vec<ListItem> = app.monsters.iter().map(|m| {
        let style = if m.is_slain {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default()
        };
        let status = if m.is_slain { " [X] " } else { " [ ] " };
        ListItem::new(format!("{}{}", status, m.name)).style(style)
    }).collect();

    app.selection_state.select(Some(app.cursor_pos));

    let list = List::new(monsters)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" SELECT TARGET | Time Left: {} ", app.get_time_left()))
            .style(Style::default().fg(Color::Yellow)))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_widget(Clear, target_area); // Clear the background
    f.render_stateful_widget(list, target_area, &mut app.selection_state);
}

pub fn render_march(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    // 1. CASTLE ART (Top of Map Area)
    let castle_ascii = crate::ui::castle::CASTLE_ASCII;
    f.render_widget(
        Paragraph::new(castle_ascii)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow)),
        chunks[0]
    );

    let map_area = chunks[1];
    let knight_idx = app.get_knight_path_index();
    let knight_pos = app.map.path_steps[knight_idx];
    let (kx, ky) = knight_pos;

    let view_height = map_area.height as u16;
    let view_width = map_area.width as u16;

    // Center viewport vertically on knight
    let mut start_y = ky.saturating_sub(view_height / 2);
    if start_y + view_height > app.map.height {
        start_y = app.map.height.saturating_sub(view_height);
    }
    let end_y = (start_y + view_height).min(app.map.height);

    // Center viewport horizontally on knight
    let mut start_x = kx.saturating_sub(view_width / 2);
    if start_x + view_width > app.map.width {
        start_x = app.map.width.saturating_sub(view_width);
    }
    let end_x = (start_x + view_width).min(app.map.width);

    // Milestone indices


    let mut lines = Vec::new();
    for y in start_y..end_y {
        let mut spans = Vec::new();
        for x in start_x..end_x {
            // Knight (Purple & Bold)
            if x == kx && y == ky {
                spans.push(Span::styled(KNIGHT, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
                continue;
            }



            let cell_type = app.map.grid[y as usize][x as usize];
            
            let mut is_walked_path = false;
            if cell_type == CellType::Path {
                if y > ky || (y == ky && app.map.path_steps[..knight_idx].contains(&(x, y))) {
                    is_walked_path = true;
                }
            }

            let (icon, color, modifier) = match cell_type {
                CellType::Tree => ("t", Color::Green, Modifier::empty()),
                CellType::Mountain => ("^", Color::White, Modifier::BOLD),
                CellType::River => ("≈", Color::Blue, Modifier::empty()),
                CellType::Bridge => ("═", Color::Indexed(94), Modifier::BOLD),
                CellType::Path => {
                    if is_walked_path {
                        (".", Color::Indexed(237), Modifier::empty()) // Walked path looks like grass
                    } else {
                        ("*", Color::Indexed(94), Modifier::BOLD) // Unwalked trail is brown '*'
                    }
                }
                CellType::Grass => (".", Color::Indexed(237), Modifier::empty()),
            };

            spans.push(Span::styled(icon, Style::default().fg(color).add_modifier(modifier)));
        }
        lines.push(Line::from(spans));
    }

    let map_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Adventure Map | Time Left: {} ", app.get_time_left()));
    
    let map = Paragraph::new(lines)
        .block(map_block)
        .alignment(Alignment::Center);
    f.render_widget(map, map_area);
}


pub fn render_footer(f: &mut Frame, app: &mut App, area: Rect) {
    let mut keys = String::from("[S] Slay | [P] Pause | [Q] Quit");
    
    // Add the explicit completion option only when quest is finished
    if app.is_quest_finished() {
        keys = format!("{} | [C] COMPLETE QUEST", keys);
    }

    let status = if app.all_monsters_slain() {
        format!("ALL FOES VANQUISHED. YOU MAY COMPLETE THE QUEST.")
    } else if app.is_goal_reached() {
        format!("YOU HAVE REACHED THE CASTLE. YOU MAY COMPLETE THE QUEST.")
    } else {
        format!("THE MARCH CONTINUES...")
    };

    let footer = Paragraph::new(format!("{}\n{}", status, keys))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
        
    f.render_widget(footer, area);
}

pub fn render_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // 1. BOUNTY BOARD
    let monsters: Vec<ListItem> = app.monsters.iter().map(|m| {
        let style = if m.is_slain {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let symbol = if m.is_slain { SLAIN } else { MONSTER };
        let symbol_style = if m.is_slain { 
            Style::default().fg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) 
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", symbol), symbol_style),
            Span::styled(&m.name, style),
        ]))
    }).collect();

    let bounty_list = List::new(monsters)
        .block(Block::default().borders(Borders::ALL).title(" Quest Log "));
    f.render_widget(bounty_list, chunks[0]);

    // 2. LEGEND
    let legend_text = vec![
        Line::from(vec![Span::styled(format!(" {} ", KNIGHT), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)), Span::raw("You (The Knight)")]),
        Line::from(vec![Span::styled(format!(" {} ", MONSTER), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)), Span::raw("Monster (Bounty)")]),
        Line::from(vec![Span::styled(format!(" {} ", SLAIN), Style::default().fg(Color::DarkGray)), Span::raw("Slain Foe")]),
        Line::from(""),
        Line::from(vec![Span::styled(" ^ ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)), Span::raw("Mountain")]),
        Line::from(vec![Span::styled(" t ", Style::default().fg(Color::Green)), Span::raw("Tree")]),
        Line::from(vec![Span::styled(" ≈ ", Style::default().fg(Color::Blue)), Span::raw("Water")]),
        Line::from(""),
        Line::from(vec![Span::styled(" * ", Style::default().fg(Color::Indexed(94))), Span::raw("The Trail")]),
    ];

    let legend = Paragraph::new(legend_text)
        .block(Block::default().borders(Borders::ALL).title(" Legend "));
    f.render_widget(legend, chunks[1]);
}

fn draw_paused_overlay(f: &mut Frame) {
    let area = f.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Length(3),
            Constraint::Percentage(45),
        ])
        .split(area);
    
    let target_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(vertical[1])[1];

    let paused_text = Paragraph::new("[ PAUSED ]")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK))
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Yellow)));

    f.render_widget(Clear, target_area);
    f.render_widget(paused_text, target_area);
}