use crate::config::Config;
use crate::ripgrep::{RipgrepBuilder, SearchResult};
use anyhow::Result;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Search,
    Help,
    PresetMenu,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    SearchInput,
    Results,
    Preview,
}

pub struct AppState {
    pub config: Config,
    pub mode: Mode,
    pub focus: Focus,
    
    // Search state
    pub search_input: String,
    pub current_path: String,
    pub file_types: Vec<String>,
    pub ignore_case: bool,
    pub last_search_time: Option<Instant>,
    pub auto_search_delay_ms: u64,
    
    // Results
    pub results: Vec<SearchResult>,
    pub selected_result: usize,
    pub results_scroll: usize,
    
    // UI state
    pub show_help: bool,
    pub help_scroll: usize,
    pub show_preset_menu: bool,
    pub selected_preset: usize,
    pub preset_names: Vec<String>,
    
    // Status
    pub status_message: String,
    pub search_time_ms: u64,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let preset_names: Vec<String> = config.presets.keys().cloned().collect();
        
        Self {
            config,
            mode: Mode::Normal,
            focus: Focus::SearchInput,
            search_input: String::new(),
            current_path: ".".to_string(),
            file_types: Vec::new(),
            ignore_case: false,
            last_search_time: None,
            auto_search_delay_ms: 500,
            results: Vec::new(),
            selected_result: 0,
            results_scroll: 0,
            show_help: false,
            help_scroll: 0,
            show_preset_menu: false,
            selected_preset: 0,
            preset_names,
            status_message: "Press 'i' or '/' to search, '?' for help, 'q' to quit".to_string(),
            search_time_ms: 0,
        }
    }

    // Mode transitions
    pub fn enter_search_mode(&mut self) {
        self.mode = Mode::Search;
        self.focus = Focus::SearchInput;
        self.status_message = "Type to search... (Enter to search, Esc to cancel)".to_string();
    }

    pub fn exit_search_mode(&mut self) {
        self.mode = Mode::Normal;
        self.focus = Focus::Results;
        self.update_status();
    }

    pub fn toggle_help(&mut self) {
        if self.mode == Mode::Help {
            self.mode = Mode::Normal;
            self.show_help = false;
        } else {
            self.mode = Mode::Help;
            self.show_help = true;
        }
    }

    pub fn toggle_preset_menu(&mut self) {
        if self.mode == Mode::PresetMenu {
            self.mode = Mode::Normal;
            self.show_preset_menu = false;
        } else {
            self.mode = Mode::PresetMenu;
            self.show_preset_menu = true;
            self.selected_preset = 0;
        }
    }

    // Search input
    pub fn add_char_to_search(&mut self, c: char) {
        self.search_input.push(c);
        self.last_search_time = Some(Instant::now());
    }

    pub fn remove_char_from_search(&mut self) {
        self.search_input.pop();
        self.last_search_time = Some(Instant::now());
    }

    pub fn clear_search(&mut self) {
        self.search_input.clear();
        self.last_search_time = Some(Instant::now());
    }

    // Search execution
    pub fn execute_search(&mut self) -> Result<()> {
        if self.search_input.is_empty() {
            self.results.clear();
            self.selected_result = 0;
            self.results_scroll = 0;
            return Ok(());
        }

        let start = Instant::now();
        
        let mut builder = RipgrepBuilder::new(self.search_input.clone())
            .path(self.current_path.clone())
            .ignore_case(self.ignore_case)
            .max_results(self.config.search.max_results);

        for ft in &self.file_types {
            builder = builder.file_type(ft.clone());
        }

        match builder.execute() {
            Ok(results) => {
                self.search_time_ms = start.elapsed().as_millis() as u64;
                self.results = results;
                self.selected_result = 0;
                self.results_scroll = 0;
                self.update_status();
                self.last_search_time = None;
            }
            Err(e) => {
                self.status_message = format!("Search error: {}", e);
                self.last_search_time = None;
            }
        }

        Ok(())
    }

    pub fn should_auto_search(&self) -> bool {
        if let Some(last_time) = self.last_search_time {
            if self.mode == Mode::Search {
                let elapsed = last_time.elapsed().as_millis() as u64;
                return elapsed >= self.auto_search_delay_ms;
            }
        }
        false
    }

    // Navigation
    pub fn next_result(&mut self) {
        if !self.results.is_empty() {
            self.selected_result = (self.selected_result + 1).min(self.results.len() - 1);
            // Auto-scroll: if selected is near bottom, scroll down
            if self.selected_result > self.results_scroll + 20 {
                self.results_scroll = self.selected_result.saturating_sub(10);
            }
        }
    }

    pub fn prev_result(&mut self) {
        if self.selected_result > 0 {
            self.selected_result -= 1;
            // Auto-scroll: if selected goes above visible area, scroll up
            if self.selected_result < self.results_scroll {
                self.results_scroll = self.selected_result;
            }
        }
    }

    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::SearchInput => Focus::Results,
            Focus::Results => Focus::Preview,
            Focus::Preview => Focus::SearchInput,
        };
    }

    pub fn prev_focus(&mut self) {
        self.focus = match self.focus {
            Focus::SearchInput => Focus::Preview,
            Focus::Results => Focus::SearchInput,
            Focus::Preview => Focus::Results,
        };
    }

    pub fn scroll_help_down(&mut self) {
        self.help_scroll += 1;
    }

    pub fn scroll_help_up(&mut self) {
        if self.help_scroll > 0 {
            self.help_scroll -= 1;
        }
    }

    // Preset management
    pub fn next_preset(&mut self) {
        if !self.preset_names.is_empty() {
            self.selected_preset = (self.selected_preset + 1) % self.preset_names.len();
        }
    }

    pub fn prev_preset(&mut self) {
        if !self.preset_names.is_empty() {
            if self.selected_preset == 0 {
                self.selected_preset = self.preset_names.len() - 1;
            } else {
                self.selected_preset -= 1;
            }
        }
    }

    pub fn apply_selected_preset(&mut self) -> Result<()> {
        if let Some(preset_name) = self.preset_names.get(self.selected_preset) {
            if let Some(preset) = self.config.presets.get(preset_name) {
                self.file_types = preset.file_types.clone();
                self.ignore_case = preset.ignore_case;
                
                self.status_message = format!("Applied preset: {}", preset_name);
                
                if !self.search_input.is_empty() {
                    self.execute_search()?;
                }
            }
        }
        Ok(())
    }

    // Actions
    pub fn open_selected_result(&mut self) -> Result<()> {
        if let Some(result) = self.results.get(self.selected_result) {
            self.status_message = format!(
                "Opening: {}:{}",
                result.path.display(),
                result.line_number
            );
            // TODO: Implement file opening in external editor
        }
        Ok(())
    }

    fn update_status(&mut self) {
        let count = self.results.len();
        if count == 0 {
            self.status_message = "No results found".to_string();
        } else {
            self.status_message = format!(
                "{} result{} found in {}ms",
                count,
                if count == 1 { "" } else { "s" },
                self.search_time_ms
            );
        }
    }
}
