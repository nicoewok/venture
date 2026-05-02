use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

#[derive(Clone, Copy, Default, PartialEq)]
pub enum CellType {
    #[default]
    Grass,
    Path,
    Tree,
    Mountain,
    River,
    Bridge,
}

pub struct WorldMap {
    pub width: u16,
    pub height: u16,
    pub grid: Vec<Vec<CellType>>,
    pub path_steps: Vec<(u16, u16)>,
}

pub enum Animation {
    Slash { frame: usize },
}

impl WorldMap {
    pub fn new(width: u16, height: u16) -> Self {
        let mut grid = vec![vec![CellType::Grass; width as usize]; height as usize];
        
        let mut connected_path = Vec::new();
        
        // Start at bottom center
        let mut walk_y = height as i32 - 1;
        let mut walk_x = (width / 2) as i32;
        
        // End at top center
        let target_x = (width / 2) as i32;
        let target_y = 0;

        connected_path.push((walk_x as u16, walk_y as u16));
        grid[walk_y as usize][walk_x as usize] = CellType::Path;

        while walk_y > target_y {
            let total_dist = (height as i32 - 1) - target_y;
            let current_dist = (height as i32 - 1) - walk_y;
            let progress = current_dist as f32 / total_dist as f32;
            
            // Base sine wave for a winding path
            let amplitude = 15.0; // Constant amplitude
            let freq = 0.2;
            let wave_offset = (amplitude * (walk_y as f32 * freq).sin()) as i32;
            
            // Linear interpolation from start_x to target_x
            let start_base_x = (width / 2) as i32;
            let base_x = start_base_x + ((target_x - start_base_x) as f32 * progress) as i32;
            
            let desired_x = base_x + wave_offset;
            let desired_x = desired_x.clamp(2, (width as i32) - 3);

            let dist_x = desired_x - walk_x;
            
            if dist_x != 0 {
                if dist_x > 0 { walk_x += 1; } else { walk_x -= 1; }
            } else {
                walk_y -= 1;
            }
            
            connected_path.push((walk_x as u16, walk_y as u16));
            grid[walk_y as usize][walk_x as usize] = CellType::Path;
        }

        // Add rivers
        for y_base in (15..height as usize - 5).step_by(30) {
            let offset_seed = y_base as f32 * 0.1;
            for x in 0..width as usize {
                let y_offset = (2.0 * (x as f32 * 0.15 + offset_seed).sin()) as i32;
                let y = (y_base as i32 + y_offset).clamp(0, height as i32 - 1) as usize;
                
                if grid[y][x] == CellType::Path {
                    grid[y][x] = CellType::Bridge;
                } else if grid[y][x] == CellType::Grass {
                    grid[y][x] = CellType::River;
                }
            }
        }

        // Add some basic terrain (trees and mountains) procedurally with clustering
        for y in 0..height as usize {
            for x in 0..width as usize {
                if grid[y][x] != CellType::Grass { continue; }
                
                let val = (x.wrapping_mul(37) ^ y.wrapping_mul(101)) % 100;
                
                // Tree clusters
                let tree_cluster = ((x / 3).wrapping_mul(13) ^ (y / 3).wrapping_mul(7)) % 10;
                if tree_cluster < 3 && val < 40 {
                    grid[y][x] = CellType::Tree;
                }
                
                // Mountain clusters
                let mtn_cluster = ((x / 5).wrapping_mul(17) ^ (y / 5).wrapping_mul(11)) % 10;
                if mtn_cluster < 2 && val < 30 {
                    grid[y][x] = CellType::Mountain;
                }
            }
        }

        Self { width, height, grid, path_steps: connected_path }
    }
}

use crate::app::{App, Scene};

const KNIGHT: &str = "K";
const MONSTER: &str = "M";
const SLAIN: &str = "X";

pub fn render(f: &mut Frame, app: &App) {
    match app.state {
        Scene::Setup => draw_setup(f, app),
        Scene::Tavern => draw_tavern(f, app),
        Scene::Naming => draw_naming(f, app),
        Scene::March => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(if f.area().width > 100 {
                    [Constraint::Percentage(75), Constraint::Percentage(25)]
                } else {
                    [Constraint::Percentage(100), Constraint::Percentage(0)]
                })
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
        Scene::Battle(idx) => render_battle(f, app, idx),
        Scene::Selection => {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(if f.area().width > 100 {
                    [Constraint::Percentage(75), Constraint::Percentage(25)]
                } else {
                    [Constraint::Percentage(100), Constraint::Percentage(0)]
                })
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
}

pub fn draw_setup(f: &mut Frame, app: &App) {
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

pub fn draw_tavern(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(11), // Tavern ASCII
            Constraint::Min(10),    // Bounty Board
            Constraint::Length(3),  // Instructions
        ])
        .split(f.area());

    let tavern_ascii = r#"
             _   _
            ( )_( )
             |   |
          ____|___|____
         |             |
         |  THE RUSTY   |
         |    ANVIL     |
         |   [  _  ]    |
         |    | | |     |
    _____|____|_|_|_____|_____
    "#;

    let title = Paragraph::new(tavern_ascii)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(title, chunks[0]);

    let mut items: Vec<ListItem> = Vec::new();

    // 1. Dotdo Tasks
    for (i, task) in app.available_tasks.iter().enumerate() {
        let style = if i == app.cursor_pos {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let tagged = if app.selected_indices.contains(&i) { " [X] " } else { " [ ] " };
        items.push(ListItem::new(format!("{}{}", tagged, task.title)).style(style));
    }

    // 2. Custom Monsters
    let tasks_len = app.available_tasks.len();
    for (i, name) in app.custom_monsters.iter().enumerate() {
        let idx = tasks_len + i;
        let style = if idx == app.cursor_pos {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        items.push(ListItem::new(format!(" [X] {}", name)).style(style));
    }

    // 3. [+] Button
    let custom_len = app.custom_monsters.len();
    let plus_idx = tasks_len + custom_len;
    let plus_style = if app.cursor_pos == plus_idx {
        Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(Color::Green)
    };
    items.push(ListItem::new(" [+] Add Custom Monster").style(plus_style));

    // 4. [START QUEST] Button
    let start_idx = plus_idx + 1;
    let selected_count = app.selected_indices.len() + custom_len;
    let can_start = selected_count == app.monsters_goal as usize;
    
    let start_style = if app.cursor_pos == start_idx {
        if can_start {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED | Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::REVERSED)
        }
    } else {
        if can_start {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };
    
    let start_text = if can_start {
        " [!] START QUEST ".to_string()
    } else if selected_count > app.monsters_goal as usize {
        format!(" [ ] TOO MANY MONSTERS (Selected: {})", selected_count)
    } else {
        format!(" [ ] START QUEST (Need {} more)", app.monsters_goal as usize - selected_count)
    };
    items.push(ListItem::new(start_text).style(start_style));

    let bounty_board = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(" Bounty Board (Goal: {}) ", app.monsters_goal)))
        .highlight_symbol("> ");
    
    f.render_widget(bounty_board, chunks[1]);

    let instructions = Paragraph::new("[Up/Down] Navigate | [Space/Enter] Select/Action | [Q] Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(instructions, chunks[2]);
}

pub fn draw_naming(f: &mut Frame, app: &App) {
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

pub fn draw_selection(f: &mut Frame, app: &App) {
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

    let monsters: Vec<ListItem> = app.monsters.iter().enumerate().map(|(i, m)| {
        let style = if i == app.cursor_pos {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED)
        } else if m.is_slain {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default()
        };
        let status = if m.is_slain { " [X] " } else { " [ ] " };
        ListItem::new(format!("{}{}", status, m.name)).style(style)
    }).collect();

    let list = List::new(monsters)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" SELECT TARGET | Time Left: {} ", app.get_time_left()))
            .style(Style::default().fg(Color::Yellow)))
        .highlight_symbol("> ");

    f.render_widget(Clear, target_area); // Clear the background
    f.render_widget(list, target_area);
}

pub fn render_march(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    // 1. CASTLE ART (Top of Map Area)
    let castle_ascii = r#"
             / \
            |   |
         _-'     '-_
        |___________|
         |         |
     ____|_________|____
    |                   |
    |  _   _     _   _  |
    | | |_| |   | |_| | |
    | |_____|   |_____| |
    |  |   |     |   |  |
    |__|___|_____|___|__|
    "#;
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
    
    let map = Paragraph::new(lines).block(map_block);
    f.render_widget(map, map_area);
}



pub fn render_battle(f: &mut Frame, app: &App, monster_idx: usize) {
    let monster = &app.monsters[monster_idx];
    let area = f.area();
    
    let monster_arts = vec![
        r#"
             (  )   (  )
              ) (   ) (
             (   ) (   )
              \  | |  /
               \ | | /
              _(_|_|_)_
             (_________)
        "#,
        r#"
              / \__
             (    @\___
             /         O
            /   (_____/
           /_____/   U
        "#,
        r#"
             .-"```"-.
            /         \
            | @   @   |
            |   ^     |
             \  -    /
              '-...-'
        "#,
        r#"
              _______
             /       \
            |  O   O  |
            |    V    |
             \_______/
              /     \
        "#,
    ];

    let monster_art = monster_arts[monster.art_idx % monster_arts.len()];

    let mut art_lines: Vec<Line> = Vec::new();
    let rows: Vec<&str> = monster_art.lines().collect();

    for (y, row) in rows.iter().enumerate() {
        let mut spans = Vec::new();
        for (x, ch) in row.chars().enumerate() {
            let mut style = Style::default();
            let mut final_char = ch.to_string();

            if let Some(crate::ui::Animation::Slash { frame, .. }) = app.animation {
                // Animated Slash: Diagonal line of blocks moving across
                let slash_pos = (frame * 3) as i32; // Adjusted speed
                let dx = x as i32;
                let dy = (y * 3) as i32; // Adjusted vertical spread
                
                if (dx - slash_pos).abs() < 2 && (dy - (dx * 2)).abs() < 15 {
                     style = style.fg(Color::White).add_modifier(Modifier::BOLD);
                     final_char = "█".to_string();
                } else {
                     style = style.fg(Color::Red);
                }
            } else if monster.is_slain {
                style = style.fg(Color::DarkGray);
            }

            spans.push(Span::styled(final_char, style));
        }
        art_lines.push(Line::from(spans));
    }

    let text = format!(
        "\n\nYOU ARE FIGHTING: {} [{}]\n\n[S] SLAY | [F] FLEE | [Q] QUIT",
        monster.name.to_uppercase(),
        monster.monster_type.to_uppercase()
    );

    let mut final_content = art_lines;
    final_content.push(Line::from(text));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" BATTLE ")
        .style(if app.animation.is_some() { Style::default().fg(Color::Red) } else { Style::default() });

    f.render_widget(
        Paragraph::new(final_content)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}


pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut keys = String::from("[S] Slay | [P] Pause | [Q] Quit");
    
    // Add the explicit completion option only when quest is finished
    if app.is_quest_finished() {
        keys = format!("{} | [C] COMPLETE QUEST", keys);
    }

    let status = if app.all_monsters_slain() {
        format!("{}  ALL FOES VANQUISHED. YOU MAY COMPLETE THE QUEST.", KNIGHT)
    } else if app.is_goal_reached() {
        format!("{}  YOU HAVE REACHED THE CITADEL. THE GATES ARE OPEN.", KNIGHT)
    } else {
        format!("{}  THE MARCH CONTINUES...", KNIGHT)
    };

    let footer = Paragraph::new(format!("{}\n{}", status, keys))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
        
    f.render_widget(footer, area);
}

pub fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
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