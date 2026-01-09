use anyhow::Result;
use clap::{Parser, Subcommand};

mod app;
mod config;
mod ripgrep;
mod ui;

#[derive(Parser)]
#[command(name = "ripgrep-tui")]
#[command(author, version, about = "A user-friendly TUI wrapper for ripgrep", long_about = None)]
struct Cli {
    /// Search pattern
    #[arg(value_name = "PATTERN")]
    pattern: Option<String>,

    /// Path to search in
    #[arg(value_name = "PATH", default_value = ".")]
    path: Option<String>,

    /// Launch interactive TUI mode
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// Use a saved preset
    #[arg(short = 'p', long = "preset")]
    preset: Option<String>,

    /// File types to include (e.g., rust, python, js)
    #[arg(short = 't', long = "type")]
    file_types: Vec<String>,

    /// Case insensitive search
    #[arg(short = 'I', long = "ignore-case")]
    ignore_case: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the interactive TUI
    Interactive,
    
    /// Manage presets
    Preset {
        #[command(subcommand)]
        action: PresetAction,
    },
    
    /// Show ripgrep options with examples
    Options {
        /// Search for specific option
        query: Option<String>,
    },
}

#[derive(Subcommand)]
enum PresetAction {
    /// List all presets
    List,
    /// Create a new preset
    New { name: String },
    /// Delete a preset
    Delete { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = config::Config::load()?;

    // Determine mode
    if cli.interactive || cli.command.as_ref().map_or(false, |c| matches!(c, Commands::Interactive)) {
        // Launch TUI
        app::run_tui(config)?;
    } else if let Some(Commands::Preset { action }) = cli.command {
        handle_preset_command(action, &config)?;
    } else if let Some(Commands::Options { query }) = cli.command {
        show_help(query)?;
    } else if let Some(pattern) = cli.pattern {
        // Direct search mode with enhanced output
        let mut rg = ripgrep::RipgrepBuilder::new(pattern)
            .path(cli.path.unwrap_or_else(|| ".".to_string()))
            .ignore_case(cli.ignore_case);

        for ft in cli.file_types {
            rg = rg.file_type(ft);
        }

        if let Some(preset_name) = cli.preset {
            rg = rg.apply_preset(&config, &preset_name)?;
        }

        let results = rg.execute()?;
        ui::display_results(&results)?;
    } else {
        // No pattern provided, launch interactive mode
        app::run_tui(config)?;
    }

    Ok(())
}

fn handle_preset_command(action: PresetAction, config: &config::Config) -> Result<()> {
    match action {
        PresetAction::List => {
            println!("Available presets:");
            for (name, preset) in &config.presets {
                println!("  {} - {}", name, preset.description);
            }
        }
        PresetAction::New { name } => {
            println!("Creating preset '{}' - feature coming soon!", name);
        }
        PresetAction::Delete { name } => {
            println!("Deleting preset '{}' - feature coming soon!", name);
        }
    }
    Ok(())
}

fn show_help(query: Option<String>) -> Result<()> {
    if let Some(q) = query {
        println!("Searching ripgrep options for: {}", q);
        // TODO: Implement option search
    } else {
        println!("Ripgrep TUI - Common Options:");
        println!("\nSearch Options:");
        println!("  -i, --ignore-case     Case insensitive search");
        println!("  -w, --word-regexp     Match whole words");
        println!("  -x, --line-regexp     Match whole lines");
        println!("\nFilter Options:");
        println!("  -t, --type            Filter by file type");
        println!("  -g, --glob            Include/exclude with glob");
        println!("\nFor full interactive help, run: ripgrep-tui --interactive");
    }
    Ok(())
}
