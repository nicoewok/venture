use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw_tavern(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(11), // Tavern ASCII
            Constraint::Length(9),  // Bounty Board (fixed height for 5-6 items)
            Constraint::Min(3),     // Instructions / Extra space
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
        let style = Style::default();
        let tagged = if app.selected_indices.contains(&i) { " [X] " } else { " [ ] " };
        items.push(ListItem::new(format!("{}{}", tagged, task.title)).style(style));
    }

    // 2. Custom Monsters
    let tasks_len = app.available_tasks.len();
    for (_, name) in app.custom_monsters.iter().enumerate() {
        let style = Style::default();
        items.push(ListItem::new(format!(" [X] {}", name)).style(style));
    }

    // 3. [+] Button
    let custom_len = app.custom_monsters.len();
    let _plus_idx = tasks_len + custom_len;
    let plus_style = Style::default().fg(Color::Green);
    items.push(ListItem::new(" [+] Add Custom Monster").style(plus_style));

    // 4. [START QUEST] Button
    let selected_count = app.selected_indices.len() + custom_len;
    let can_start = selected_count == app.monsters_goal as usize;
    
    let start_style = if can_start {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    let start_text = if can_start {
        " [!] START QUEST ".to_string()
    } else if selected_count > app.monsters_goal as usize {
        format!(" [ ] TOO MANY MONSTERS (Selected: {})", selected_count)
    } else {
        format!(" [ ] START QUEST (Need {} more)", app.monsters_goal as usize - selected_count)
    };
    items.push(ListItem::new(start_text).style(start_style));

    app.tavern_state.select(Some(app.cursor_pos));

    let mut bounty_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Bounty Board (Goal: {}) ", app.monsters_goal));
    
    if items.len() > 7 {
        bounty_block = bounty_block.title_bottom(Line::from(" [...] ").alignment(Alignment::Center));
    }

    let bounty_board = List::new(items)
        .block(bounty_block)
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");
    
    f.render_stateful_widget(bounty_board, chunks[1], &mut app.tavern_state);

    let instructions = Paragraph::new("[Up/Down] Navigate | [Space/Enter] Select/Action | [Q] Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(instructions, chunks[2]);
}
