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
                          _  _
    _ _      (0)(0)-._  _.-'^^'^^'^^'^^'^^'--.
   (.(.)----'`        ^^'                /^   ^^-._
   (    `                 \             |    _    ^^-._
    VvvvvvvVv~~`__,/.._>  /:/:/:/:/:/:/:/\  (_..,______^^-.
     `^^^^^^^^`/  /   /  /`^^^^^^^^^>^^>^`>  >        _`)  )
              (((`   (((`          (((`  (((`        `'--'^
              
        Alligator
        "#,
        r#"
                           .
                          / V\
                        / `  /
                       <<   |
                       /    |
                     /      |
                   /        |
                 /    \  \ /
                (      ) | |
        ________|   _/_  | |
      <__________\______)\__)
                
                Wolf
        "#,
        r#"
             .-"```"-.
            /         \
            | 0   0   |
            |   ^     |
            \  -    /
             '-...-'
              
              Skeleton
        "#,
        r#"
                  _______      
             /       \
            |  O   O  |
            |    V    |
             \_______/
            /     \

                Slime
        "#,
        r#"
                  ^    ^
                 / \  //\
   |\___/|      /   \//  .\
   /O  O  \__  /    //  | \ \
  /     /  \/_/    //   |  \  \
  @___@'    \/_   //    |   \   \
     |       \/_ //     |    \    \
     |        \///      |     \     \
    _|_/   )  //       |      \     _\
              /,___/  ( ; -.    |    _ _\.-~      .-~~~^-.
              -{       _      `-.|.-~-.        .~         `.
               /\     /                 ~-..-~     .-~^-.  \
                  `.   {            }                   /\  \
                .----~-.\        \-'               .~     \  `. \^-.
               ///.----..>    c   \          _ -~          `.  ^-`   ^-_
                 ///-._______}^     -------~                 ~--,   .-~
                                                                /.-'
                         Dragon
        "#,
    ];

pub fn render_battle(f: &mut Frame, app: &mut App, monster_idx: usize) {
    let monster = &app.monsters[monster_idx];
    let area = f.area();
    
    let monster_art_raw = MONSTER_ARTS[monster.art_idx % MONSTER_ARTS.len()];

    // Clean up art: trim empty lines and strip common indentation
    let all_lines: Vec<&str> = monster_art_raw.lines().collect();
    let first = all_lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
    let last = all_lines.iter().rposition(|l| !l.trim().is_empty()).unwrap_or(all_lines.len().saturating_sub(1));
    let lines = &all_lines[first..=last];

    let min_indent = lines.iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    let rows: Vec<String> = lines.iter()
        .map(|l| if l.len() >= min_indent { l[min_indent..].to_string() } else { l.to_string() })
        .collect();

    let max_width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let max_height = rows.len();

    let mut art_lines: Vec<Line> = Vec::new();

    for (y, row) in rows.iter().enumerate() {
        let mut spans = Vec::new();
        for (x, ch) in row.chars().enumerate() {
            let mut style = Style::default();
            let mut final_char = ch.to_string();

            let is_name_line = y == max_height - 1;

            if let Some(Animation::Slash { frame, .. }) = app.animation {
                if !is_name_line {
                    // Quicker Arch Slash: frame 0-6
                    // Sweep from left (-20) to right (max_width + 20)
                    let progress = frame as f32 / 6.0;
                    let sweep_range = max_width as f32 + 40.0;
                    let center_x = (progress * sweep_range) as i32 - 20;
                    
                    // Arch formula: curve the slash based on y position
                    let mid_y = (max_height as i32) / 2;
                    let arch_offset = ((y as i32 - mid_y).pow(2)) / 2;
                    let target_x = center_x + arch_offset;
                    
                    if (x as i32 - target_x).abs() < 3 {
                         style = style.fg(Color::White).add_modifier(Modifier::BOLD);
                         final_char = "█".to_string();
                    } else {
                         style = style.fg(Color::Red);
                    }
                }
            } else if monster.is_slain {
                style = style.fg(Color::DarkGray);
            }

            spans.push(Span::styled(final_char, style));
        }
        art_lines.push(Line::from(spans));
    }

    let mut final_content = art_lines;
    final_content.push(Line::from(""));
    final_content.push(Line::from(vec![
        Span::raw("YOU ARE FIGHTING: "),
        Span::styled(monster.name.to_uppercase(), Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
        Span::raw(format!(" [{}]", monster.monster_type.to_uppercase())),
    ]));

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
