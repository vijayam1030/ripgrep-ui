use eframe::egui;
use crate::config::Config;
use crate::ripgrep::{RipgrepBuilder, SearchResult};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct RipgrepApp {
    config: Config,
    
    // Search state
    search_pattern: String,
    search_path: String,
    
    // Options
    ignore_case: bool,
    smart_case: bool,
    hidden_files: bool,
    follow_symlinks: bool,
    word_regexp: bool,
    line_regexp: bool,
    invert_match: bool,
    
    // File filters
    selected_file_types: Vec<String>,
    available_file_types: Vec<String>,
    glob_patterns: String,
    exclude_patterns: String,
    max_depth: Option<usize>,
    max_depth_enabled: bool,
    max_depth_value: String,
    
    // Advanced options
    context_lines: usize,
    context_enabled: bool,
    max_count: Option<usize>,
    max_count_enabled: bool,
    max_count_value: String,
    multiline: bool,
    
    // Results
    results: Arc<Mutex<Vec<SearchResult>>>,
    selected_result_index: Option<usize>,
    is_searching: bool,
    search_error: Option<String>,
    search_time_ms: u64,
    result_count: usize,
    
    // UI state
    show_advanced_options: bool,
    show_preset_dialog: bool,
    selected_preset: Option<String>,
    
    // Preview
    preview_content: String,
    preview_mode: PreviewMode,
    preview_file_path: Option<std::path::PathBuf>,
    preview_line_number: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PreviewMode {
    SingleLine,
    WholeFile,
}

impl RipgrepApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        let available_file_types = vec![
            "rust", "python", "js", "ts", "java", "cpp", "c", "go", "php", 
            "ruby", "swift", "kotlin", "cs", "html", "css", "json", "yaml",
            "xml", "md", "txt", "sh", "sql", "r", "scala", "perl", "lua"
        ].iter().map(|s| s.to_string()).collect();

        Self {
            config,
            search_pattern: String::new(),
            search_path: ".".to_string(),
            ignore_case: false,
            smart_case: true,
            hidden_files: false,
            follow_symlinks: false,
            word_regexp: false,
            line_regexp: false,
            invert_match: false,
            selected_file_types: Vec::new(),
            available_file_types,
            glob_patterns: String::new(),
            exclude_patterns: String::new(),
            max_depth: None,
            max_depth_enabled: false,
            max_depth_value: "5".to_string(),
            context_lines: 0,
            context_enabled: false,
            max_count: None,
            max_count_enabled: false,
            max_count_value: "100".to_string(),
            multiline: false,
            results: Arc::new(Mutex::new(Vec::new())),
            selected_result_index: None,
            is_searching: false,
            search_error: None,
            search_time_ms: 0,
            result_count: 0,
            show_advanced_options: false,
            show_preset_dialog: false,
            selected_preset: None,
            preview_content: String::new(),
            preview_mode: PreviewMode::SingleLine,
            preview_file_path: None,
            preview_line_number: 0,
        }
    }

    fn execute_search(&mut self) {
        if self.search_pattern.is_empty() {
            return;
        }

        self.is_searching = true;
        self.search_error = None;
        
        let pattern = self.search_pattern.clone();
        let path = self.search_path.clone();
        let ignore_case = self.ignore_case;
        let smart_case = self.smart_case;
        let hidden = self.hidden_files;
        let file_types = self.selected_file_types.clone();
        let results_arc = Arc::clone(&self.results);
        
        thread::spawn(move || {
            let _start = std::time::Instant::now();
            
            let mut builder = RipgrepBuilder::new(pattern)
                .path(path)
                .ignore_case(ignore_case)
                .smart_case(smart_case)
                .hidden(hidden);

            for ft in file_types {
                builder = builder.file_type(ft);
            }

            match builder.execute() {
                Ok(search_results) => {
                    let mut results = results_arc.lock().unwrap();
                    *results = search_results;
                }
                Err(e) => {
                    eprintln!("Search error: {}", e);
                }
            }
        });
    }

    fn load_whole_file_preview(&mut self) {
        if let Some(path) = &self.preview_file_path {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    // Limit file size to prevent UI freeze
                    if content.len() > 500_000 {
                        self.preview_content = format!(
                            "File too large to preview ({}KB). Showing first 500KB...\n\n{}",
                            content.len() / 1024,
                            &content[..500_000]
                        );
                    } else {
                        self.preview_content = content;
                    }
                }
                Err(e) => {
                    self.preview_content = format!("Error reading file: {}", e);
                }
            }
        }
    }
}

impl eframe::App for RipgrepApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if search is complete
        if self.is_searching {
            if let Ok(results) = self.results.try_lock() {
                self.result_count = results.len();
                self.is_searching = false;
            }
            ctx.request_repaint();
        }

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📁 Open Folder...").clicked() {
                        // TODO: Open folder dialog
                        ui.close_menu();
                    }
                    if ui.button("💾 Save Results...").clicked() {
                        // TODO: Save results
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🚪 Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("Presets", |ui| {
                    for (name, preset) in &self.config.presets {
                        if ui.button(format!("📦 {}", name)).clicked() {
                            self.selected_file_types = preset.file_types.clone();
                            self.ignore_case = preset.ignore_case;
                            ui.close_menu();
                        }
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("📖 Documentation").clicked() {
                        // TODO: Open docs
                        ui.close_menu();
                    }
                    if ui.button("ℹ️ About").clicked() {
                        // TODO: Show about
                        ui.close_menu();
                    }
                });
            });
        });

        // Left panel - Search options
        egui::SidePanel::left("search_panel")
            .min_width(300.0)
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.heading("🔍 Search Options");
                ui.separator();
                
                ui.add_space(10.0);
                
                // Search pattern
                ui.label("Pattern:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.search_pattern)
                        .hint_text("Enter search pattern (regex supported)")
                        .desired_width(f32::INFINITY)
                );
                
                if response.changed() {
                    // Auto-search on change (with small delay)
                }
                
                ui.add_space(5.0);
                
                // Search path
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_path)
                            .hint_text("./")
                            .desired_width(ui.available_width() - 60.0)
                    );
                    if ui.button("📁").clicked() {
                        // TODO: Open folder picker
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                // Basic options
                ui.label("🎯 Match Options:");
                ui.checkbox(&mut self.ignore_case, "Ignore case (-i)");
                ui.checkbox(&mut self.smart_case, "Smart case (-S)");
                ui.checkbox(&mut self.word_regexp, "Match whole words (-w)");
                ui.checkbox(&mut self.line_regexp, "Match whole lines (-x)");
                ui.checkbox(&mut self.invert_match, "Invert match (-v)");
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                
                // File options
                ui.label("📁 File Options:");
                ui.checkbox(&mut self.hidden_files, "Include hidden files");
                ui.checkbox(&mut self.follow_symlinks, "Follow symlinks");
                
                ui.add_space(5.0);
                
                // File types
                ui.label("File Types:");
                egui::ScrollArea::vertical()
                    .id_source("file_types_scroll")
                    .max_height(150.0)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for ft in &self.available_file_types.clone() {
                                let selected = self.selected_file_types.contains(ft);
                                if ui.selectable_label(selected, ft).clicked() {
                                    if selected {
                                        self.selected_file_types.retain(|x| x != ft);
                                    } else {
                                        self.selected_file_types.push(ft.clone());
                                    }
                                }
                            }
                        });
                    });
                
                ui.add_space(10.0);
                
                // Advanced options toggle
                if ui.button(if self.show_advanced_options { "▼ Hide Advanced" } else { "▶ Show Advanced" }).clicked() {
                    self.show_advanced_options = !self.show_advanced_options;
                }
                
                if self.show_advanced_options {
                    ui.separator();
                    ui.add_space(10.0);
                    
                    ui.label("⚙️ Advanced Options:");
                    
                    // Max depth
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.max_depth_enabled, "Max depth:");
                        ui.add_enabled(
                            self.max_depth_enabled,
                            egui::TextEdit::singleline(&mut self.max_depth_value)
                                .desired_width(50.0)
                        );
                    });
                    
                    // Max count
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.max_count_enabled, "Max results:");
                        ui.add_enabled(
                            self.max_count_enabled,
                            egui::TextEdit::singleline(&mut self.max_count_value)
                                .desired_width(50.0)
                        );
                    });
                    
                    ui.checkbox(&mut self.multiline, "Multiline matching");
                    
                    ui.add_space(5.0);
                    
                    // Glob patterns
                    ui.label("Include globs:");
                    ui.text_edit_singleline(&mut self.glob_patterns);
                    
                    ui.label("Exclude patterns:");
                    ui.text_edit_singleline(&mut self.exclude_patterns);
                }
                
                ui.add_space(20.0);
                
                // Search button
                let search_button = egui::Button::new(if self.is_searching { "⏳ Searching..." } else { "🔍 Search" })
                    .min_size(egui::vec2(ui.available_width(), 40.0));
                
                if ui.add_enabled(!self.is_searching && !self.search_pattern.is_empty(), search_button).clicked() {
                    self.execute_search();
                }
                
                ui.add_space(10.0);
                
                // Status
                if self.is_searching {
                    ui.spinner();
                    ui.label("Searching...");
                } else if let Some(error) = &self.search_error {
                    ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
                } else if self.result_count > 0 {
                    ui.colored_label(
                        egui::Color32::GREEN,
                        format!("✅ Found {} result{} in {}ms", 
                            self.result_count,
                            if self.result_count == 1 { "" } else { "s" },
                            self.search_time_ms
                        )
                    );
                }
            });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📊 Search Results");
            ui.separator();
            
            // Results table
            use egui_extras::{Column, TableBuilder};
            
            let results = self.results.lock().unwrap().clone();
            
            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(60.0))    // Line number
                .column(Column::remainder())     // File path
                .column(Column::exact(80.0))    // Actions
                .header(25.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Line");
                    });
                    header.col(|ui| {
                        ui.strong("File");
                    });
                    header.col(|ui| {
                        ui.strong("Actions");
                    });
                })
                .body(|mut body| {
                    for (idx, result) in results.iter().enumerate() {
                        body.row(25.0, |mut row| {
                            row.col(|ui| {
                                ui.label(result.line_number.to_string());
                            });
                            row.col(|ui| {
                                let label = ui.selectable_label(
                                    self.selected_result_index == Some(idx),
                                    result.path.display().to_string()
                                );
                                if label.clicked() {
                                    self.selected_result_index = Some(idx);
                                    self.preview_file_path = Some(result.path.clone());
                                    self.preview_line_number = result.line_number;
                                    
                                    // Update preview based on mode
                                    if self.preview_mode == PreviewMode::SingleLine {
                                        self.preview_content = format!(
                                            "File: {}\nLine: {}\n\n{}",
                                            result.path.display(),
                                            result.line_number,
                                            result.line_content
                                        );
                                    } else {
                                        // Load whole file
                                        self.load_whole_file_preview();
                                    }
                                }
                            });
                            row.col(|ui| {
                                if ui.button("📂 Open").clicked() {
                                    // TODO: Open file in editor
                                }
                            });
                        });
                    }
                });
            
            ui.add_space(10.0);
            
            // Preview panel
            if !self.preview_content.is_empty() {
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.heading("👁️ Preview");
                    ui.add_space(20.0);
                    
                    ui.label("Mode:");
                    if ui.selectable_label(self.preview_mode == PreviewMode::SingleLine, "📄 Single Line").clicked() {
                        self.preview_mode = PreviewMode::SingleLine;
                        // Reload preview in single line mode
                        if let Some(idx) = self.selected_result_index {
                            let results = self.results.lock().unwrap();
                            if let Some(result) = results.get(idx) {
                                self.preview_content = format!(
                                    "File: {}\nLine: {}\n\n{}",
                                    result.path.display(),
                                    result.line_number,
                                    result.line_content
                                );
                            }
                        }
                    }
                    if ui.selectable_label(self.preview_mode == PreviewMode::WholeFile, "📑 Whole File").clicked() {
                        self.preview_mode = PreviewMode::WholeFile;
                        self.load_whole_file_preview();
                    }
                });
                
                egui::ScrollArea::both()
                    .id_source("preview_scroll")
                    .min_scrolled_height(200.0)
                    .max_height(300.0)
                    .show(ui, |ui| {
                        if self.preview_mode == PreviewMode::WholeFile {
                            // Display whole file with line numbers and highlighting
                            egui::Grid::new("file_preview_grid")
                                .num_columns(2)
                                .spacing([10.0, 2.0])
                                .striped(false)
                                .show(ui, |ui| {
                                    for (line_num, line) in self.preview_content.lines().enumerate() {
                                        let actual_line = line_num + 1;
                                        
                                        // Line number
                                        if actual_line == self.preview_line_number {
                                            ui.colored_label(egui::Color32::YELLOW, format!("{:>4}", actual_line));
                                        } else {
                                            ui.colored_label(egui::Color32::DARK_GRAY, format!("{:>4}", actual_line));
                                        }
                                        
                                        // Content
                                        if actual_line == self.preview_line_number {
                                            ui.horizontal(|ui| {
                                                ui.colored_label(egui::Color32::YELLOW, "➤");
                                                ui.colored_label(egui::Color32::from_rgb(255, 255, 150), line);
                                            });
                                        } else {
                                            ui.label(line);
                                        }
                                        
                                        ui.end_row();
                                    }
                                });
                        } else {
                            // Single line mode - no columns or separators
                            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.monospace(&self.preview_content);
                            });
                        }
                    });
            }
        });
    }
}
