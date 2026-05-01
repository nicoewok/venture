use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, ListItem, List, Clear},
    style::{Style, Color, Modifier},
    text::{Line, Span},
    Frame,
};
use crate::app::{App, Scene};

const KNIGHT: &str = "\u{f15d1} "; // 󱗑
const SWORDS: &str = "\u{2694} ";  // ⚔
const MONSTER: &str = "\u{f1719} "; // 󱜙

pub fn render(f: &mut Frame, app: &App) {
    match app.state {
        Scene::Setup => draw_setup(f, app),
        Scene::Tavern => draw_tavern(f, app),
        Scene::March => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(3)])
                .split(f.area());
            render_march(f, app, chunks[0]);
            render_footer(f, app, chunks[1]);
        }
        Scene::Battle(idx) => render_battle(f, app, idx),
        Scene::Selection => {
            // Render March background
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(3)])
                .split(f.area());
            render_march(f, app, chunks[0]);
            render_footer(f, app, chunks[1]);
            
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

    let mut items: Vec<ListItem> = app.available_tasks.iter().enumerate().map(|(i, task)| {
        let style = if i == app.cursor_pos {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let tagged = if app.selected_indices.contains(&i) { " [X] " } else { " [ ] " };
        ListItem::new(format!("{}{}", tagged, task.title)).style(style)
    }).collect();

    // Add the "+" item for custom monsters
    let plus_style = if app.cursor_pos == app.available_tasks.len() {
        Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(Color::Green)
    };
    let plus_text = if app.custom_monster_count > 0 {
        format!(" [+] Add Custom Monster (Stored: {})", app.custom_monster_count)
    } else {
        " [+] Add Custom Monster".to_string()
    };
    items.push(ListItem::new(plus_text).style(plus_style));

    let bounty_board = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Bounty Board "))
        .highlight_symbol("> ");
    
    f.render_widget(bounty_board, chunks[1]);

    let instructions = Paragraph::new("[Up/Down] Navigate | [Space] Tag Monster | [Enter] Start Quest | [Q] Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(instructions, chunks[2]);
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
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(area);

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[0]);

    // Top Right: Slain Count & Timer
    let stats = format!("{} Slain: {} | 󱎫 {}", KNIGHT, app.slain_count, app.get_time_left());
    f.render_widget(
        Paragraph::new(stats).alignment(Alignment::Right),
        header_chunks[1]
    );

    // Castle Art in chunks[0]
    let castle_ascii = r#"
             |
           [[ ]]
      _____|_____|_____
     |                 |
     |   _   _   _     |
     |  | |_| |_| |    |
     |__|_________|____|
    "#;
    f.render_widget(Paragraph::new(castle_ascii).alignment(Alignment::Center), chunks[0]);

    // 2. DRAW THE WINDING TRAIL
    let mut path_render = String::new();
    let knight_row_idx = 18; // Row 25 of terminal (7 for castle + 18)

    for (i, row) in app.trail.iter().enumerate() {
        let left_border = "/";
        let right_border = "\\";
        let width = app.path_width;

        // Start with the space before the path, but inject left decoration if it exists
        let mut line = String::new();
        let left_dec_pos = (row.center as i32 - 6).max(0) as usize;
        
        line.push_str(&" ".repeat(left_dec_pos));
        if let Some(ref dec) = row.left_decoration {
            line.push_str(dec);
            line.push_str(&" ".repeat(row.center - left_dec_pos - 1));
        } else {
            line.push_str(&" ".repeat(row.center - left_dec_pos));
        }
        
        if i == knight_row_idx {
            // Place Knight
            line.push_str(left_border);
            let padding = (width.saturating_sub(2)) / 2;
            line.push_str(&" ".repeat(padding));
            line.push_str(KNIGHT);
            line.push_str(&" ".repeat(width.saturating_sub(padding).saturating_sub(2)));
            line.push_str(right_border);
        } else if let Some(ref icon) = row.content {
            // Place Monster/Landmark
            line.push_str(left_border);
            let padding = (width.saturating_sub(2)) / 2;
            line.push_str(&" ".repeat(padding));
            line.push_str(icon);
            line.push_str(&" ".repeat(width.saturating_sub(padding).saturating_sub(2)));
            line.push_str(right_border);
        } else {
            // Empty Path
            line.push_str(left_border);
            line.push_str(&" ".repeat(width));
            line.push_str(right_border);
        }

        // Add right decoration if it exists
        if let Some(ref dec) = row.right_decoration {
            line.push_str(&" ".repeat(6));
            line.push_str(dec);
        }
        
        path_render.push_str(&line);
        path_render.push('\n');
    }

    f.render_widget(
        Paragraph::new(path_render).alignment(Alignment::Left),
        chunks[1]
    );
}



pub fn render_battle(f: &mut Frame, app: &App, monster_idx: usize) {
    let monster = &app.monsters[monster_idx];
    
    let area = f.area();
    let monster_art = r#"
             (  )   (  )
              ) (   ) (
             (   ) (   )
              \  | |  /
               \ | | /
              _(_|_|_)_
             (_________)
    "#; // A simple ASCII "Blob" or "Ghost"

    let text = format!(
        "\n\nYOU ARE FIGHTING: {}\n\n[S] SLAY | [F] FLEE | [Q] QUIT",
        monster.name.to_uppercase()
    );

    f.render_widget(
        Paragraph::new(format!("{}\n{}", monster_art, text))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" BATTLE ")),
        area,
    );
}


pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut keys = String::from("[S] Slay | [P] Pause | [Q] Quit");
    
    // Add the explicit completion option only at the destination
    if app.is_goal_reached() {
        keys = format!("{} | [C] COMPLETE QUEST", keys);
    }

    let status = if app.is_goal_reached() {
        "󱗑  YOU HAVE REACHED THE CITADEL. THE GATES ARE OPEN."
    } else {
        "󱗑  THE MARCH CONTINUES..."
    };

    let footer = Paragraph::new(format!("{}\n{}", status, keys))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
        
    f.render_widget(footer, area);
}