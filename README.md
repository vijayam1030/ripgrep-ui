# Ripgrep TUI

A user-friendly Terminal UI wrapper for [ripgrep](https://github.com/BurntSushi/ripgrep) with enhanced discoverability and easy-to-use functionality.

![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg)

## Features

✨ **Interactive TUI Mode** - Beautiful terminal interface with live search results  
🔍 **Real-time Search** - See results as you type  
🎨 **Syntax Highlighting** - Color-coded matches and file previews  
📋 **Preset Management** - Save and reuse common search configurations  
⚡ **Blazing Fast** - Powered by ripgrep's speed  
🎯 **Easy Discovery** - Built-in help and guided options  
🖥️ **Cross-platform** - Works on Windows, Linux, and macOS

## Prerequisites

You must have [ripgrep](https://github.com/BurntSushi/ripgrep) installed:

### Windows
```bash
# Using Chocolatey
choco install ripgrep

# Using Scoop
scoop install ripgrep

# Using winget
winget install BurntSushi.ripgrep.MSVC
```

### Linux
```bash
# Ubuntu/Debian
sudo apt install ripgrep

# Fedora
sudo dnf install ripgrep

# Arch
sudo pacman -S ripgrep
```

### macOS
```bash
brew install ripgrep
```

## Installation

### From Source
```bash
git clone <repository-url>
cd ripgrep
cargo build --release
cargo install --path .
```

The binary will be available as `ripgrep-tui`.

## Usage

### Interactive Mode (Default)
Launch the TUI interface:
```bash
ripgrep-tui
# or
ripgrep-tui --interactive
```

### Direct Search Mode
Search directly from command line with enhanced output:
```bash
ripgrep-tui "pattern" [PATH]
ripgrep-tui "TODO" src/
ripgrep-tui "function" . -t rust
```

### Using Presets
```bash
# Apply a preset
ripgrep-tui "pattern" -p rust

# List available presets
ripgrep-tui preset list
```

## Interactive Mode Controls

### Navigation
- `i` or `/` - Enter search mode
- `j` or `↓` - Move down in results
- `k` or `↑` - Move up in results  
- `Tab` - Switch focus between panels
- `Enter` - Open selected result
- `q` - Quit application

### Search
- `p` - Open preset menu
- `Ctrl+U` - Clear search input
- `Esc` - Cancel search/close menus

### Help
- `?` - Toggle help panel

## Configuration

Configuration file location:
- **Windows**: `%APPDATA%\ripgrep-tui\config.toml`
- **Linux/macOS**: `~/.config/ripgrep-tui/config.toml`

### Example Configuration
```toml
[ui]
theme = "dark"
syntax_highlighting = true
preview_lines = 5

[search]
ignore_case = false
smart_case = true
max_results = 1000

# Custom preset
[presets.myproject]
description = "Search my project files"
file_types = ["rust", "toml", "md"]
ignore_case = false
hidden = false
glob = []
max_depth = 5
additional_args = []
```

## Built-in Presets

### `rust`
Search in Rust source files only
- File types: `*.rs`

### `code`
Search in common programming files
- File types: `.rs`, `.py`, `.js`, `.ts`, `.go`, `.cpp`, `.c`, `.h`

### `docs`
Search in documentation files
- File types: `.md`, `.txt`, `.rst`
- Case insensitive

## Command Line Options

```
Usage: ripgrep-tui [OPTIONS] [PATTERN] [PATH]

Arguments:
  [PATTERN]  Search pattern (regex supported)
  [PATH]     Path to search in [default: .]

Options:
  -i, --interactive          Launch interactive TUI mode
  -p, --preset <PRESET>      Use a saved preset
  -t, --type <TYPE>          File types to include (e.g., rust, python, js)
  -I, --ignore-case          Case insensitive search
  -h, --help                 Print help
  -V, --version              Print version

Subcommands:
  interactive    Launch the interactive TUI
  preset         Manage presets (list, new, delete)
  help           Show ripgrep options with examples
```

## Examples

### Interactive Mode
```bash
# Launch TUI
ripgrep-tui

# Type pattern → real-time results
# Press 'p' → select preset
# Navigate results → preview files
```

### Command Line
```bash
# Basic search
ripgrep-tui "TODO" src/

# Search with file type filter
ripgrep-tui "error" . -t rust

# Case insensitive search
ripgrep-tui "function" . -I

# Use preset
ripgrep-tui "main" -p rust

# List presets
ripgrep-tui preset list
```

## Architecture

```
ripgrep-tui/
├── src/
│   ├── main.rs           # Entry point, CLI parsing
│   ├── app.rs            # TUI application logic
│   │   ├── state.rs      # Application state management
│   │   └── ui_render.rs  # UI rendering with ratatui
│   ├── ripgrep.rs        # Ripgrep wrapper and execution
│   ├── config.rs         # Configuration management
│   └── ui.rs             # CLI output formatting
├── Cargo.toml            # Dependencies
└── README.md
```

## Technology Stack

- **Language**: Rust 2021 edition
- **TUI Framework**: [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- **Terminal Backend**: [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- **CLI Parsing**: [clap](https://github.com/clap-rs/clap) - Command line argument parser
- **Search Engine**: [ripgrep](https://github.com/BurntSushi/ripgrep) - Fast recursive search

## Development

### Build
```bash
cargo build
```

### Run
```bash
cargo run -- --interactive
cargo run -- "pattern" src/
```

### Test
```bash
cargo test
```

### Release Build
```bash
cargo build --release
```

## Roadmap

- [ ] File opening in external editor (VS Code, Vim, etc.)
- [ ] Export results (JSON, HTML, Markdown)
- [ ] Custom syntax highlighting themes
- [ ] Search history and bookmarks
- [ ] Replace functionality
- [ ] Multi-pattern search
- [ ] Advanced regex builder
- [ ] Performance metrics and profiling

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- [ripgrep](https://github.com/BurntSushi/ripgrep) by Andrew Gallant - The amazing search tool that powers this project
- [ratatui](https://github.com/ratatui-org/ratatui) - Excellent TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal library

## Support

If you encounter any issues or have questions:
1. Check if ripgrep is installed: `rg --version`
2. Check the configuration file for errors
3. Run with `--help` for usage information
4. Open an issue on GitHub

---

Made with ❤️ and Rust
