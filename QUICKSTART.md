# Quick Start Guide

## Installation

1. **Install Ripgrep** (required dependency):
   ```bash
   # Windows (Chocolatey)
   choco install ripgrep
   
   # Windows (Scoop)
   scoop install ripgrep
   
   # Windows (winget)
   winget install BurntSushi.ripgrep.MSVC
   ```

2. **Build Ripgrep TUI**:
   ```bash
   cd ripgrep
   cargo build --release
   ```

3. **Install locally**:
   ```bash
   cargo install --path .
   ```

## First Run

Launch the interactive TUI:
```bash
ripgrep-tui
```

You'll see a beautiful interface with:
- **Search bar** at the top
- **Results panel** on the left
- **Preview panel** on the right
- **Status bar** at the bottom

## Basic Usage

1. **Start Searching**:
   - Press `i` or `/` to enter search mode
   - Type your search pattern
   - Results appear in real-time!

2. **Navigate Results**:
   - Use `j`/`k` or arrow keys to move
   - See live preview on the right

3. **Use Presets**:
   - Press `p` to open preset menu
   - Select a preset (e.g., "rust" for Rust files only)
   - Your search is automatically filtered

4. **Get Help**:
   - Press `?` to see all shortcuts
   - Press `?` again to close

5. **Exit**:
   - Press `q` to quit

## Command Line Mode

Skip the TUI and search directly:
```bash
# Basic search
ripgrep-tui "TODO" src/

# Search in specific file types
ripgrep-tui "function" . -t rust

# Case insensitive
ripgrep-tui "error" . -I

# Use preset
ripgrep-tui "main" -p code
```

## Tips

- **Real-time search**: Results update as you type (500ms delay)
- **Regex support**: Use regex patterns just like ripgrep
- **Smart case**: Automatically case-insensitive for lowercase patterns
- **Custom presets**: Edit `~/.config/ripgrep-tui/config.toml` to add your own

## Next Steps

Check out the [README.md](README.md) for:
- Full feature list
- Configuration options
- All keyboard shortcuts
- Creating custom presets
