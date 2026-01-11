# Glyphgen - Terminal Art Rendering Studio

[![CI](https://github.com/JayabrataBasu/Glyphgen/actions/workflows/ci.yml/badge.svg)](https://github.com/JayabrataBasu/Glyphgen/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Terminal User Interface (TUI) application for converting images to ASCII/Unicode art and stylizing text with Unicode fonts. Built with Rust for maximum performance and cross-platform compatibility.

## Features

### Image to ASCII Art
- Convert images (PNG, JPEG, GIF, WebP, BMP) to ASCII art
- Multiple character sets: Standard, Extended, Unicode Blocks
- Adjustable output width
- Invert and edge enhancement options
- Real-time preview

### Image to Unicode Art
- High-fidelity Unicode rendering with color support
- Multiple modes:
  - **Blocks**: Simple block characters (░▒▓█)
  - **Half-Blocks**: 2x vertical resolution using ▀▄
  - **Braille**: 2x4 resolution using Braille patterns
- Full color support (16, 256, and TrueColor)
- Automatic terminal capability detection

### Text Stylizer
- Convert plain text to stylized Unicode
- 14 Unicode styles including:
  - Bold (𝐀𝐁𝐂), Italic (𝐴𝐵𝐶), Bold Italic (𝑨𝑩𝑪)
  - Script (𝒜ℬ𝒞), Fraktur (𝔄𝔅ℭ), Double-Struck (𝔸𝔹ℂ)
  - Sans-Serif (𝖠𝖡𝖢), Monospace (𝙰𝙱𝙲)
  - Fullwidth (ＡＢＣ), Circled (ⒶⒷⒸ), and more
- Gradient coloring (horizontal, rainbow)

### Performance
- 60 FPS UI with non-blocking rendering
- Multi-threaded rendering on worker threads
- Sub-100ms render times for typical images
- Zero GC pauses (native Rust)

## Installation

### From Releases

Download the latest release for your platform from the [Releases page](https://github.com/JayabrataBasu/Glyphgen/releases).

### From Source

```bash
# Clone the repository
git clone https://github.com/JayabrataBasu/Glyphgen.git
cd Glyphgen

# Build release binary
cargo build --release

# Run
./target/release/glyphgen
```

### Requirements

- Rust 1.75+ (for building from source)
- A terminal with UTF-8 support
- Recommended: A terminal with TrueColor support (e.g., Kitty, Alacritty, iTerm2, Windows Terminal)

## Usage

### Quick Start

```bash
# Run the application
glyphgen

# Load an image using environment variable
GLYPHGEN_IMAGE=path/to/image.png glyphgen

# Load an image via command line
glyphgen --image path/to/image.png

# Render once and save (batch mode)
glyphgen --image photo.jpg --render-once --mode unicode --output-format png
```

#### CLI Options

| Option | Description |
|--------|-------------|
| `--image PATH` | Load image from path |
| `--render-once` | Render immediately and exit (batch mode) |
| `--mode MODE` | Render mode: `ascii`, `unicode`, or `text` |
| `--output-format FMT` | Output format: `ansi`, `html`, `txt`, `png`, `svg` |

### Keyboard Controls

#### Global
| Key | Action |
|-----|--------|
| `Q` | Quit application |
| `?` | Toggle help overlay |
| `Tab` | Next widget |
| `Shift+Tab` | Previous widget |
| `Esc` | Cancel / Close overlay |

#### Mode Selector
| Key | Action |
|-----|--------|
| `1`, `2`, `3` | Jump to mode |
| `↑` `↓` | Navigate modes |
| `Enter` | Select mode |

#### Control Panel
| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate settings |
| `←` `→` | Adjust selection |
| `+` `-` | Adjust numeric values |
| `Space` | Toggle / Render |
| `L` | Load image |
| `S` | Save output |

#### Preview Area
| Key | Action |
|-----|--------|
| `↑` `↓` | Scroll by line |
| `PgUp` `PgDn` | Scroll by page |
| `Home` `End` | Jump to top/bottom |
| `C` | Copy to clipboard |
| `S` | Save output |

### Output Formats

Glyphgen supports multiple output formats for saving your art:

| Format | Extension | Description | ASCII Mode | Unicode Mode |
|--------|-----------|-------------|------------|--------------|
| **ANSI** | `.ansi` | Raw ANSI escape codes (terminal compatible) | ✓ | ✓ |
| **HTML** | `.html` | HTML with inline CSS colors | ✓ | ✓ |
| **TXT** | `.txt` | Plain text without colors | ✓ | ✗ |
| **PNG** | `.png` | Rasterized image with bundled font | ✓ | ✓ |
| **SVG** | `.svg` | Vector graphics with text elements | ✓ | ✓ |

**Note:** TXT format is excluded from Unicode mode because Unicode block characters rely on colors for proper display.

To change the output format:
- Press `O` to cycle through available formats
- Or adjust "Output Format" in the Control Panel using `←` `→`

### Configuration

Configuration is automatically saved to:
- Linux: `~/.config/glyphgen/config.toml`
- macOS: `~/Library/Application Support/glyphgen/config.toml`
- Windows: `%APPDATA%\glyphgen\config.toml`

Example configuration:

```toml
[ascii]
default_charset = "Extended"
default_width = 80
edge_enhance = false

[unicode]
default_mode = "HalfBlocks"
default_width = 80

[text]
default_style = "Bold"
default_gradient = "None"

[ui]
show_line_numbers = false
word_wrap = false
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Layer                        │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────────┐  │
│  │  Event Loop    │  │  State Manager │  │  UI Renderer     │  │
│  │  (Main Thread) │  │  (Main Thread) │  │  (Main Thread)   │  │
│  └────────────────┘  └────────────────┘  └──────────────────┘  │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            │ Channel-based Message Passing
                            │
┌───────────────────────────┴─────────────────────────────────────┐
│                      Processing Layer (Worker Threads)           │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │ ASCII Engine     │  │ Unicode Engine   │  │ Text Stylizer │ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Rust + Ratatui + Crossterm**: Battle-tested stack for high-performance TUI applications
2. **Multi-threaded Architecture**: Rendering happens on worker threads to maintain 60 FPS UI
3. **Message Passing**: Zero shared mutable state, communication via channels
4. **Zero-Copy Images**: `Arc<DynamicImage>` for efficient image sharing

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure

```
src/
├── main.rs              # Entry point, event loop
├── lib.rs               # Library exports
├── state.rs             # Application state management
├── input.rs             # Keyboard input handling
├── worker.rs            # Background worker threads
├── config.rs            # Configuration management
├── image_loader.rs      # Image loading utilities
├── color_space.rs       # Color conversion
├── unicode_handler.rs   # Unicode width/validation
├── terminal_capabilities.rs
├── perf_monitor.rs      # Performance tracking
├── ui/
│   ├── mod.rs           # Main UI rendering
│   ├── help.rs          # Help overlay
│   ├── preview.rs       # Preview area
│   └── widgets.rs       # Control panel widgets
└── render_engines/
    ├── mod.rs
    ├── ascii.rs         # ASCII art renderer
    ├── unicode.rs       # Unicode art renderer
    └── text_stylizer.rs # Text stylization
```

## Performance

Typical render times on modern hardware:

| Operation | Image Size | Output Width | Time |
|-----------|------------|--------------|------|
| ASCII | 800×600 | 80 | ~20ms |
| ASCII | 1920×1080 | 120 | ~35ms |
| Unicode (HalfBlocks) | 800×600 | 80 | ~40ms |
| Unicode (TrueColor) | 800×600 | 80 | ~60ms |
| Text Stylize | 100 chars | - | <1ms |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI library
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [image](https://github.com/image-rs/image) - Rust image processing library

---

# Glyphgen
An Ascii generator
