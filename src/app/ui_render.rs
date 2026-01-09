use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use crate::app::state::{AppState, Focus, Mode};

pub fn render(f: &mut Frame, app: &AppState) {
    if app.show_help {
        render_help(f, app);
    } else if app.show_preset_menu {
        render_preset_menu(f, app);
    } else {
        render_main(f, app);
    }
}

fn render_main(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Search input
            Constraint::Min(10),     // Results + Preview
            Constraint::Length(3),  // Status bar
        ])
        .split(f.size());

    // Header
    render_header(f, chunks[0], app);

    // Search input
    render_search_input(f, chunks[1], app);

    // Split results and preview horizontally
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Results
            Constraint::Percentage(50),  // Preview
        ])
        .split(chunks[2]);

    render_results(f, main_chunks[0], app);
    render_preview(f, main_chunks[1], app);

    // Status bar
    render_status_bar(f, chunks[3], app);
}

fn render_header(f: &mut Frame, area: Rect, _app: &AppState) {
    let title = vec![
        Span::styled("⚡ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Ripgrep", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled(" TUI", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" ⚡", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("Fast Search with Style", Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC)),
    ];

    let subtitle = if let Ok(version) = crate::ripgrep::get_ripgrep_version() {
        format!(" 🔍 {} ", version)
    } else {
        " 🔍 ripgrep ".to_string()
    };

    let header = Paragraph::new(Line::from(title))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta))
            .title(subtitle))
        .alignment(Alignment::Center);

    f.render_widget(header, area);
}

fn render_search_input(f: &mut Frame, area: Rect, app: &AppState) {
    let is_focused = app.focus == Focus::SearchInput || app.mode == Mode::Search;
    
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let input_text = if app.search_input.is_empty() {
        Span::styled("✨ Type pattern to search...", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
    } else {
        Span::styled(&app.search_input, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
    };

    let mut title = vec![
        Span::styled("🔍 Search ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    ];
    
    if !app.file_types.is_empty() {
        title.push(Span::styled(
            format!("📁[{}] ", app.file_types.join(", ")),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ));
    }
    
    if app.ignore_case {
        title.push(Span::styled("[Aa] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
    }

    let input = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Line::from(title))
            .border_style(border_style));

    f.render_widget(input, area);

    // Show cursor when in search mode
    if app.mode == Mode::Search {
        f.set_cursor(area.x + app.search_input.len() as u16 + 1, area.y + 1);
    }
}

fn render_results(f: &mut Frame, area: Rect, app: &AppState) {
    let is_focused = app.focus == Focus::Results;
    
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let title = format!("📋 Results ({}) ", app.results.len());

    let items: Vec<ListItem> = app.results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let is_selected = i == app.selected_result;
            
            let path_str = result.path.display().to_string();
            let short_path = if path_str.len() > 40 {
                format!("...{}", &path_str[path_str.len() - 37..])
            } else {
                path_str
            };

            let line = vec![
                Span::styled(
                    format!("{:>4} ", result.line_number),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                ),
                Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    short_path.clone(),
                    Style::default().fg(Color::Cyan),
                ),
            ];

            let style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(line)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD));

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_result));

    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_preview(f: &mut Frame, area: Rect, app: &AppState) {
    let is_focused = app.focus == Focus::Preview;
    
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Magenta)
    };

    let content = if let Some(result) = app.results.get(app.selected_result) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("📄 File: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    result.path.display().to_string(),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
        ];

        // Show the matched line with highlighting
        let line_content = &result.line_content;
        let mut spans = Vec::new();
        let mut last_pos = 0;

        for m in &result.matches {
            // Add text before match
            if m.start > last_pos {
                spans.push(Span::raw(&line_content[last_pos..m.start]));
            }
            // Add highlighted match
            spans.push(Span::styled(
                &line_content[m.start..m.end],
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
            last_pos = m.end;
        }

        // Add remaining text
        if last_pos < line_content.len() {
            spans.push(Span::raw(&line_content[last_pos..]));
        }

        lines.push(Line::from(spans));
        
        Text::from(lines)
    } else {
        Text::from("No result selected")
    };

    let preview = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("👁️  Preview ")
            .border_style(border_style))
        .wrap(Wrap { trim: false });

    f.render_widget(preview, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &AppState) {
    let help_text = match app.mode {
        Mode::Normal => "⌨️  i//:search | ?:help | p:presets | j/k:navigate | Enter:open | q:quit",
        Mode::Search => "✏️  Type to search | Enter:execute | Esc:cancel | Ctrl+U:clear",
        Mode::Help => "📖 j/k:scroll | ?/Esc:close",
        Mode::PresetMenu => "📑 j/k:navigate | Enter:select | Esc:cancel",
    };

    let status_style = if app.results.is_empty() && !app.search_input.is_empty() {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if !app.results.is_empty() {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let status = Paragraph::new(vec![
        Line::from(Span::styled(&app.status_message, status_style)),
        Line::from(Span::styled(help_text, Style::default().fg(Color::Yellow))),
    ])
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta)));

    f.render_widget(status, area);
}

fn render_help(f: &mut Frame, app: &AppState) {
    let area = centered_rect(80, 80, f.size());

    let help_text = vec![
        Line::from(vec![
            Span::styled("⚡ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Ripgrep TUI ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("- Help Guide", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("🎯 Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  i, /", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("           Enter search mode"),
        ]),
        Line::from(vec![
            Span::styled("  j, Down", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("        Move down in results"),
        ]),
        Line::from(vec![
            Span::styled("  k, Up", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("          Move up in results"),
        ]),
        Line::from(vec![
            Span::styled("  Tab", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("            Switch focus between panels"),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("          Open selected result"),
        ]),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("              Quit application"),
        ]),
        Line::from(""),
        Line::from(Span::styled("🔍 Search Options:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  p", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw("              Open preset menu"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+U", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw("         Clear search input"),
        ]),
        Line::from(""),
        Line::from(Span::styled("📦 Presets:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  rust", Style::default().fg(Color::Cyan)),
            Span::raw("           Search in Rust files only"),
        ]),
        Line::from(vec![
            Span::styled("  code", Style::default().fg(Color::Cyan)),
            Span::raw("           Search in common code files"),
        ]),
        Line::from(vec![
            Span::styled("  docs", Style::default().fg(Color::Cyan)),
            Span::raw("           Search in documentation files"),
        ]),
        Line::from(""),
        Line::from(Span::styled("✨ Features:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(vec![Span::styled("  ⚡ ", Style::default().fg(Color::Yellow)), Span::raw("Real-time search as you type")]),
        Line::from(vec![Span::styled("  🎨 ", Style::default().fg(Color::Yellow)), Span::raw("Colorful syntax-highlighted results")]),
        Line::from(vec![Span::styled("  👁️  ", Style::default().fg(Color::Yellow)), Span::raw("File preview panel")]),
        Line::from(vec![Span::styled("  ⚙️  ", Style::default().fg(Color::Yellow)), Span::raw("Customizable presets")]),
        Line::from(vec![Span::styled("  🚀 ", Style::default().fg(Color::Yellow)), Span::raw("Fast ripgrep backend")]),
        Line::from(""),
        Line::from(Span::styled("💡 Press '?' or Esc to close this help", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" 📚 Help ")
            .border_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
        .wrap(Wrap { trim: false })
        .scroll((app.help_scroll as u16, 0));

    f.render_widget(help, area);
}

fn render_preset_menu(f: &mut Frame, app: &AppState) {
    let area = centered_rect(60, 50, f.size());

    let items: Vec<ListItem> = app.preset_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_selected = i == app.selected_preset;
            
            let description = app.config.presets
                .get(name)
                .map(|p| p.description.as_str())
                .unwrap_or("");

            let icon = match name.as_str() {
                "rust" => "🦀 ",
                "code" => "💻 ",
                "docs" => "📝 ",
                _ => "📦 ",
            };

            let line = Line::from(vec![
                Span::raw(icon),
                Span::styled(name.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" → ", Style::default().fg(Color::DarkGray)),
                Span::styled(description.to_string(), Style::default().fg(Color::Green)),
            ]);

            let style = if is_selected {
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" 🎨 Select Preset ")
            .border_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
