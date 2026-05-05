use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::Animation;

const MONSTER_ARTS: [&str; 5] = [
        r#"
              ___    ___
             /   \__/   \
            |  [O] [O]  |
            |    _V_    |
             \  |___|  /
             /         \
            /   [XXX]   \
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
            | 0   0   |
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
        r#"
                     -,,,__
                     \    ``~~--,,__                /   /
                     /              ``~~--,,_     //--//
          _,,,,-----,\              ,,,,---- >   (c  c)\
      ,;''            `\,,,,----''''   ,,-'''---/   /_ ;___        -,_
     ( ''---,;====;,----/             (-,,_____/  /'/ `;   '''''----\ `:.
     (                 '               `      (oo)/   ;~~~~~~~~~~~~~/--~
      `;_           ;    \            ;   \   `  ' ,,'
         ```-----...|     )___________|    )-----'''
                     \   /             \   \\
                     /  /,              `\   \\
                   ,'---\ \              ,---`,;,
                         ```
                    Dragon
        "#,
    ];

pub fn render_battle(f: &mut Frame, app: &mut App, monster_idx: usize) {
    let monster = &app.monsters[monster_idx];
    let area = f.area();
    
    let monster_art = MONSTER_ARTS[monster.art_idx % MONSTER_ARTS.len()];

    let mut art_lines: Vec<Line> = Vec::new();
    let rows: Vec<&str> = monster_art.lines().collect();

    for (y, row) in rows.iter().enumerate() {
        let mut spans = Vec::new();
        for (x, ch) in row.chars().enumerate() {
            let mut style = Style::default();
            let mut final_char = ch.to_string();

            if let Some(Animation::Slash { frame, .. }) = app.animation {
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

    let info_text = format!(
        "\n\nYOU ARE FIGHTING: {} [{}]",
        monster.name.to_uppercase(),
        monster.monster_type.to_uppercase()
    );

    let mut final_content = art_lines;
    final_content.push(Line::from(info_text));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" BATTLE ")
        .style(if app.animation.is_some() { Style::default().fg(Color::Red) } else { Style::default() });

    f.render_widget(
        Paragraph::new(final_content)
            .alignment(Alignment::Center)
            .block(block),
        chunks[0],
    );

    let footer = Paragraph::new("[S] Slay | [F] Flee | [P] Pause | [Q] Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[1]);
}
