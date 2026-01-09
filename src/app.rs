use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

mod state;
mod ui_render;

use state::AppState;
use crate::config::Config;

pub fn run_tui(config: Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Check if ripgrep is installed
    if !crate::ripgrep::check_ripgrep_installed()? {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        eprintln!("Error: ripgrep (rg) is not installed or not in PATH.");
        eprintln!("Please install ripgrep from: https://github.com/BurntSushi/ripgrep");
        eprintln!("\nOn Windows, you can install via:");
        eprintln!("  - Chocolatey: choco install ripgrep");
        eprintln!("  - Scoop: scoop install ripgrep");
        eprintln!("  - Download from releases: https://github.com/BurntSushi/ripgrep/releases");
        
        return Ok(());
    }

    // Create app state
    let mut app = AppState::new(config);

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui_render::render(f, app))?;

        // Poll for events with timeout to allow auto-search to trigger
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only process KeyPress events, ignore KeyRelease to avoid double input
                if key.kind == event::KeyEventKind::Press {
                    match app.mode {
                        state::Mode::Normal => match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('i') => app.enter_search_mode(),
                            KeyCode::Char('/') => app.enter_search_mode(),
                            KeyCode::Char('?') => app.toggle_help(),
                            KeyCode::Char('p') => app.toggle_preset_menu(),
                            KeyCode::Down | KeyCode::Char('j') => app.next_result(),
                            KeyCode::Up | KeyCode::Char('k') => app.prev_result(),
                            KeyCode::Enter => app.open_selected_result()?,
                            KeyCode::Tab => app.next_focus(),
                            KeyCode::BackTab => app.prev_focus(),
                            _ => {}
                        },
                        state::Mode::Search => match key.code {
                            KeyCode::Esc => app.exit_search_mode(),
                            KeyCode::Enter => {
                                app.execute_search()?;
                                app.exit_search_mode();
                            }
                            KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                match c {
                                    'c' => app.exit_search_mode(),
                                    'u' => app.clear_search(),
                                    _ => {}
                                }
                            }
                            KeyCode::Char(c) => app.add_char_to_search(c),
                            KeyCode::Backspace => app.remove_char_from_search(),
                            _ => {}
                        },
                        state::Mode::Help => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                                app.toggle_help();
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.scroll_help_down(),
                            KeyCode::Up | KeyCode::Char('k') => app.scroll_help_up(),
                            _ => {}
                        },
                        state::Mode::PresetMenu => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.toggle_preset_menu(),
                            KeyCode::Down | KeyCode::Char('j') => app.next_preset(),
                            KeyCode::Up | KeyCode::Char('k') => app.prev_preset(),
                            KeyCode::Enter => {
                                app.apply_selected_preset()?;
                                app.toggle_preset_menu();
                            }
                            _ => {}
                        },
                    }
                }
            }
        }

        // Auto-search mode (search as you type)
        if app.should_auto_search() {
            app.execute_search()?;
        }
    }
}
