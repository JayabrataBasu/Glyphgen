# Glyphgen: High-Performance Terminal Art Rendering Studio

**Document Version:** 1.0  
**Date:** January 9, 2026  
**Classification:** Implementation-Ready Technical Specification

---

## Executive Summary

This document specifies the complete architecture, implementation strategy, and engineering approach for a production-grade Terminal User Interface (TUI) application that functions as a terminal-based art rendering studio. The application will convert images to ASCII/Unicode art and stylize text with Unicode fonts, all with real-time preview capabilities and zero perceptible latency.

**Core Value Proposition:**
- **Performance-First:** Sub-16ms frame times, non-blocking UI, zero GC pauses during rendering
- **Production Quality:** Clean architecture, comprehensive testing, professional distribution
- **Cross-Platform:** Single binary deployment for Linux, macOS, and Windows
- **User-Centric:** Keyboard-first navigation with discoverable controls and sensible defaults

**Technology Stack (Justified):** Rust + Ratatui + Crossterm  
**Expected Timeline:** 8-12 weeks for MVP with all core rendering modes  
**Risk Level:** Low to Medium (proven stack, well-understood domain)

---

## 1. High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Layer                        │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────────┐  │
│  │  Event Loop    │  │  State Manager │  │  UI Renderer     │  │
│  │  (Main Thread) │  │  (Main Thread) │  │  (Main Thread)   │  │
│  └────────┬───────┘  └────────┬───────┘  └──────────┬───────┘  │
│           │                   │                      │           │
│           └───────────────────┴──────────────────────┘           │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            │ Channel-based Message Passing
                            │
┌───────────────────────────┴─────────────────────────────────────┐
│                      Processing Layer                            │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │ Image→ASCII      │  │ Image→Unicode    │  │ Text Stylizer │ │
│  │ Render Engine    │  │ Render Engine    │  │ Engine        │ │
│  │ (Worker Thread)  │  │ (Worker Thread)  │  │ (Worker)      │ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            │ Shared Components
                            │
┌───────────────────────────┴─────────────────────────────────────┐
│                      Foundation Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │ Image Loader │  │ Color Space  │  │ Unicode Handler    │    │
│  │ & Decoder    │  │ Converter    │  │ & Width Calculator │    │
│  └──────────────┘  └──────────────┘  └────────────────────┘    │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │ Terminal     │  │ Configuration│  │ Performance        │    │
│  │ Capability   │  │ Manager      │  │ Monitor            │    │
│  │ Detector     │  │              │  │                    │    │
│  └──────────────┘  └──────────────┘  └────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Architecture Principles

1. **Strict Thread Separation:** Main thread handles only UI and input; all rendering happens on worker threads
2. **Unidirectional Data Flow:** Events → State Updates → UI Rendering (no circular dependencies)
3. **Message-Passing Concurrency:** Rust channels for inter-thread communication (no shared mutable state)
4. **Zero-Copy Where Possible:** Use references and `Cow<str>` to minimize allocations
5. **Fail-Fast with Recovery:** Panics in worker threads must not crash the UI; graceful degradation

---

## 2. Technology Stack Justification

### Primary Language: Rust

**Decision Rationale:**
- **Performance:** Zero-cost abstractions, no GC pauses, fine-grained control over memory
- **Safety:** Compile-time guarantees eliminate data races and null pointer errors
- **Concurrency:** First-class support for fearless concurrency via ownership system
- **Ecosystem:** Mature TUI libraries (Ratatui), excellent image processing (image crate)
- **Distribution:** Produces static binaries with minimal dependencies

**Alternatives Considered and Rejected:**
- **Go:** GC pauses violate deterministic frame time requirement
- **C++:** Memory safety concerns, longer development time, complex build system
- **Python:** Too slow for real-time rendering, GIL limits concurrency
- **Zig:** Immature ecosystem, fewer libraries, steeper learning curve

### TUI Framework: Ratatui (v0.28+)

**Decision Rationale:**
- **Active Development:** Forked from tui-rs, actively maintained, modern API
- **Widget System:** Rich set of built-in widgets (List, Table, Block, Paragraph, etc.)
- **Layout Engine:** Constraint-based layout system handles responsive design
- **Performance:** Minimal overhead, efficient diffing algorithm for terminal updates
- **Community:** Strong community, extensive examples, good documentation

**API Stability:** Ratatui has stabilized; breaking changes are rare post-v0.20

### Terminal Backend: Crossterm (v0.27+)

**Decision Rationale:**
- **Cross-Platform:** Unified API for Linux, macOS, Windows (no platform-specific code)
- **Features:** Mouse support, raw mode, event handling, color support detection
- **Reliability:** Battle-tested, used in production by numerous TUI applications
- **Integration:** First-class integration with Ratatui

**Alternatives Considered:**
- **Termion:** Linux/macOS only, no Windows support
- **Termwiz:** Less mature, smaller community

### Image Processing: `image` crate (v0.25+)

**Decision Rationale:**
- **Format Support:** PNG, JPEG, GIF, BMP, WebP, TIFF out of the box
- **Pure Rust:** No C dependencies, cross-compilation friendly
- **Performance:** Efficient decoding, low memory footprint
- **API:** Ergonomic API for resizing, color conversion, pixel access

### Color Handling: `palette` crate (v0.7+)

**Decision Rationale:**
- **Color Spaces:** LAB, LCH, HSL, HSV, XYZ conversions for perceptual luminance
- **Type Safety:** Strongly-typed color representations prevent errors
- **Algorithms:** Built-in color distance metrics (Delta-E)

### Unicode Support: `unicode-width` (v0.1+) and `unicode-segmentation` (v1.10+)

**Decision Rationale:**
- **Width Calculation:** Correct East Asian Width handling (critical for alignment)
- **Grapheme Clusters:** Proper handling of combining characters and emoji
- **Standards Compliance:** Implements UAX #11 (East Asian Width) and UAX #29 (Text Segmentation)

### Concurrency: Tokio (v1.35+)

**Decision Rationale:**
- **Async Runtime:** Efficient task scheduling for I/O-bound operations (file loading)
- **Channels:** Multi-producer, single-consumer channels for worker communication
- **Ecosystem:** De facto standard for async Rust

**Note:** Rendering workers use native threads (`std::thread`), not async tasks, for CPU-bound work

---

## 3. Detailed Module Breakdown

### 3.1 Application Layer Modules

#### 3.1.1 `main.rs` - Entry Point and Event Loop

**Responsibilities:**
- Initialize terminal (raw mode, alternate screen)
- Set up panic hook for graceful recovery
- Create application state and worker threads
- Run main event loop (input → state update → render)
- Cleanup on exit (restore terminal)

**Key Functions:**
```rust
fn main() -> Result<()> {
    // Setup
    let mut terminal = setup_terminal()?;
    let (tx, rx) = mpsc::channel(); // Worker → Main communication
    let app_state = AppState::new(tx.clone());
    let workers = spawn_workers(tx);
    
    // Event loop
    loop {
        terminal.draw(|f| ui::render(f, &app_state))?;
        
        if event::poll(Duration::from_millis(16))? {
            handle_event(event::read()?, &mut app_state)?;
        }
        
        process_worker_messages(&rx, &mut app_state)?;
        
        if app_state.should_quit { break; }
    }
    
    // Cleanup
    cleanup_terminal(terminal)?;
    Ok(())
}
```

**Error Handling:**
- Panic hook captures panics and restores terminal before exit
- Worker thread panics are logged but don't crash the application
- File I/O errors are displayed as non-blocking status messages

#### 3.1.2 `state.rs` - Application State Manager

**Responsibilities:**
- Maintain single source of truth for application state
- Expose immutable getters for UI rendering
- Provide mutating methods for state transitions
- Store user preferences and last-used settings per mode

**State Structure:**
```rust
pub struct AppState {
    pub current_mode: RenderMode, // ASCII, Unicode, TextStylizer
    pub focus: FocusedWidget,     // ModeSelector, ControlPanel, Preview, Help
    
    // Mode-specific state
    pub ascii_state: AsciiRenderState,
    pub unicode_state: UnicodeRenderState,
    pub text_state: TextStylizeState,
    
    // Shared state
    pub input_file: Option<PathBuf>,
    pub preview_content: Option<String>, // Rendered output
    pub status_message: String,
    pub show_help: bool,
    pub performance_metrics: PerfMetrics,
    
    // Worker communication
    worker_tx: mpsc::Sender<WorkerMessage>,
}

pub enum RenderMode { ImageToAscii, ImageToUnicode, TextStylizer }
pub enum FocusedWidget { ModeSelector, ControlPanel, Preview, Help }

// Example mode-specific state
pub struct AsciiRenderState {
    pub charset: CharacterSet, // Standard, Extended, Custom
    pub width: usize,          // Output width in characters
    pub invert: bool,
    pub edge_enhance: bool,
    pub last_render_time_ms: u64,
}
```

**State Transition Rules:**
- All state changes go through dedicated methods (`set_mode`, `update_ascii_setting`, etc.)
- State changes that require re-rendering send messages to worker threads
- Workers send results back via channels; main thread updates `preview_content`

#### 3.1.3 `ui/mod.rs` - UI Rendering

**Responsibilities:**
- Render current state to terminal using Ratatui widgets
- Layout management (responsive to terminal size)
- Keyboard shortcut hints
- Status bar with metrics

**Layout Structure:**
```
┌─────────────────────────────────────────────────────────────┐
│ Terminal Art Studio v1.0               [?] Help  [Q] Quit   │ <- Title Bar
├──────────────┬──────────────────────────────────────────────┤
│ Mode:        │                                              │
│ [ ] ASCII    │                                              │
│ [•] Unicode  │                                              │ <- Mode Selector
│ [ ] Text     │                                              │   (Left Column)
├──────────────┤         PREVIEW AREA                         │
│ Settings:    │                                              │
│              │                                              │
│ Width: 80    │                                              │ <- Control Panel
│ Charset: ▓   │                                              │   (Left Column)
│ Color: On    │                                              │
│              │                                              │
│ [Space] Rend │                                              │
│ [L] Load Img │                                              │ <- Preview Area
│              │                                              │   (Right Column)
└──────────────┴──────────────────────────────────────────────┘
│ Status: Ready | FPS: 60 | Render: 12ms | File: image.png   │ <- Status Bar
└──────────────────────────────────────────────────────────────┘
```

**Responsive Layout:**
- Terminal width < 80 cols: Stack mode selector and controls vertically
- Terminal width >= 80 cols: Side-by-side layout as shown
- Preview area always fills remaining space
- Minimum terminal size: 40x20 (enforced with warning message)

**Rendering Function:**
```rust
pub fn render(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Min(10),    // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(f.size());
    
    render_title_bar(f, chunks[0], state);
    render_main_content(f, chunks[1], state);
    render_status_bar(f, chunks[2], state);
}
```

**Help Overlay:**
- Triggered by `?` key
- Modal overlay (dims background)
- Shows all keyboard shortcuts for current mode
- `Esc` or `?` to close

#### 3.1.4 `input.rs` - Input Handler

**Responsibilities:**
- Map key events to state transitions
- Context-sensitive key bindings (mode-dependent)
- Handle focus changes

**Key Bindings (Global):**
- `Q`: Quit application
- `?`: Toggle help overlay
- `Tab` / `Shift+Tab`: Cycle focus forward/backward
- `Arrow Keys`: Navigate within focused widget
- `Esc`: Cancel/close overlay

**Key Bindings (Context-Sensitive):**
- Mode Selector: `1`, `2`, `3` to jump to modes
- Control Panel: `+/-` to adjust numeric values, `Space` to toggle booleans
- Preview Area: `S` to save output, `C` to copy to clipboard
- File Loading: `L` to open file picker (via external command: `fzf` or native dialog)

**Input Handling Pattern:**
```rust
pub fn handle_event(event: Event, state: &mut AppState) -> Result<()> {
    match event {
        Event::Key(key_event) => {
            if state.show_help {
                return handle_help_input(key_event, state);
            }
            
            match state.focus {
                FocusedWidget::ModeSelector => handle_mode_selector_input(key_event, state),
                FocusedWidget::ControlPanel => handle_control_panel_input(key_event, state),
                FocusedWidget::Preview => handle_preview_input(key_event, state),
            }
        }
        Event::Resize(width, height) => {
            state.terminal_size = (width, height);
            Ok(())
        }
        _ => Ok(()),
    }
}
```

### 3.2 Processing Layer Modules

#### 3.2.1 `render_engines/ascii.rs` - Image to ASCII Engine

**Responsibilities:**
- Convert images to ASCII art using luminance-based character mapping
- Support multiple character sets (standard, extended, custom)
- Optional edge detection enhancement
- Adaptive downsampling based on output dimensions

**Algorithm Overview:**
1. **Load and Decode Image** (via `image` crate)
2. **Resize Image** to target dimensions (maintaining aspect ratio, accounting for character cell aspect ~2:1)
3. **Convert to Grayscale** (perceptual luminance: 0.299R + 0.587G + 0.114B)
4. **Optional Edge Detection** (Sobel operator if edge enhancement enabled)
5. **Map Luminance to Characters** (from character set based on brightness)
6. **Output as String** (row-major order with newlines)

**Character Sets:**
```rust
pub enum CharacterSet {
    Standard,   // " .:-=+*#%@"  (10 chars, low contrast)
    Extended,   // " .'`^\",:;Il!i><~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"
    Blocks,     // " ░▒▓█" (Unicode block elements)
    Custom(String), // User-provided character set (sorted by luminance)
}
```

**Edge Enhancement:**
- Compute gradient magnitude at each pixel (Sobel 3x3 kernel)
- Blend gradient with luminance (adjustable weight)
- Map combined value to character

**Performance Considerations:**
- Pre-compute luminance lookup table for character set
- Use SIMD-friendly image processing where possible (via `image` crate's optimizations)
- Allocate output string buffer once (capacity = width × height + height for newlines)

**Function Signature:**
```rust
pub fn render_ascii(
    image: &DynamicImage,
    config: &AsciiConfig,
) -> Result<String> {
    let resized = resize_for_ascii(image, config.target_width)?;
    let gray = to_grayscale(&resized);
    let chars = luminance_to_chars(&gray, &config.charset, config.invert);
    Ok(format_as_string(chars, config.target_width))
}
```

**Worker Thread Integration:**
```rust
// Worker thread spawned in main.rs
fn ascii_worker(rx: mpsc::Receiver<AsciiRequest>, tx: mpsc::Sender<WorkerResponse>) {
    while let Ok(request) = rx.recv() {
        match render_ascii(&request.image, &request.config) {
            Ok(output) => tx.send(WorkerResponse::AsciiComplete(output)).unwrap(),
            Err(e) => tx.send(WorkerResponse::Error(e.to_string())).unwrap(),
        }
    }
}
```

#### 3.2.2 `render_engines/unicode.rs` - Image to Unicode Art Engine

**Responsibilities:**
- Convert images to high-fidelity Unicode art using block characters
- Support color output (ANSI 256-color or 24-bit RGB)
- Automatic terminal capability detection
- Half-block optimization for 2x vertical resolution

**Character Sets:**
```rust
pub enum UnicodeMode {
    Blocks,       // " ░▒▓█" (4 levels)
    HalfBlocks,   // "▀▄█" + background color (2x vertical resolution)
    Braille,      // Braille patterns U+2800..U+28FF (experimental, 2x4 resolution)
}
```

**Half-Block Algorithm:**
- Each character cell represents 2 vertical pixels
- Top half: foreground color + "▀" (upper half block)
- Bottom half: background color
- Effective doubling of vertical resolution with no horizontal cost

**Color Mapping:**
- Detect terminal color support (via Crossterm's `supports_color`)
- Fallback: 256-color → 24-bit RGB → 16-color → Grayscale
- Quantize RGB to nearest terminal color (Euclidean distance in RGB space)
- Pre-build color palette lookup table

**Function Signature:**
```rust
pub fn render_unicode(
    image: &DynamicImage,
    config: &UnicodeConfig,
) -> Result<String> {
    let resized = resize_for_unicode(image, config.target_width, config.mode)?;
    
    match config.mode {
        UnicodeMode::Blocks => render_blocks(&resized, config.color_mode),
        UnicodeMode::HalfBlocks => render_half_blocks(&resized, config.color_mode),
        UnicodeMode::Braille => render_braille(&resized),
    }
}
```

**ANSI Color Embedding:**
```rust
// Example output with ANSI codes
format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, character)
```

#### 3.2.3 `render_engines/text_stylizer.rs` - Unicode Text Stylizer

**Responsibilities:**
- Convert plain text to Unicode stylized text (mathematical bold, italic, script, etc.)
- Apply gradient coloring (horizontal, vertical, per-character)
- Support multiple Unicode "fonts"
- Validate output (ensure no broken grapheme clusters)

**Unicode Styles (Mathematical Alphanumeric Symbols U+1D400..U+1D7FF):**
```rust
pub enum UnicodeStyle {
    Bold,              // 𝐀𝐁𝐂 (U+1D400)
    Italic,            // 𝐴𝐵𝐶 (U+1D434)
    BoldItalic,        // 𝑨𝑩𝑪 (U+1D468)
    Script,            // 𝒜ℬ𝒞 (U+1D49C)
    BoldScript,        // 𝓐𝓑𝓒 (U+1D4D0)
    Fraktur,           // 𝔄𝔅ℭ (U+1D504)
    DoubleStruck,      // 𝔸𝔹ℂ (U+1D538)
    SansSerif,         // 𝖠𝖡𝖢 (U+1D5A0)
    SansSerifBold,     // 𝗔𝗕𝗖 (U+1D5D4)
    Monospace,         // 𝙰𝙱𝙲 (U+1D670)
    Fullwidth,         // ＡＢＣ (U+FF21)
    Circled,           // ⒶⒷⒸ (U+24B6)
    Negative,          // 🅐🅑🅒 (U+1F150)
}
```

**Mapping Algorithm:**
- Build lookup table: ASCII char → Unicode styled char
- Handle characters with no mapping (preserve original)
- Special handling for numbers, spaces, punctuation

**Gradient Coloring:**
```rust
pub enum GradientMode {
    None,
    Horizontal { start: Rgb, end: Rgb },
    Vertical { start: Rgb, end: Rgb },
    PerCharacter { colors: Vec<Rgb> }, // Cycle through colors
}

fn apply_gradient(text: &str, mode: GradientMode) -> String {
    // Interpolate colors based on position
    // Embed ANSI color codes around each character
}
```

**Grapheme Cluster Handling:**
- Use `unicode-segmentation` to split text into grapheme clusters
- Apply styling to base characters only
- Preserve combining characters (accents, diacritics)

**Function Signature:**
```rust
pub fn stylize_text(
    text: &str,
    style: UnicodeStyle,
    gradient: GradientMode,
) -> Result<String> {
    let styled = apply_unicode_style(text, style)?;
    let colored = apply_gradient(&styled, gradient)?;
    validate_output(&colored)?; // Ensure terminal-safe
    Ok(colored)
}
```

### 3.3 Foundation Layer Modules

#### 3.3.1 `image_loader.rs` - Image Loading and Decoding

**Responsibilities:**
- Load images from disk (async, non-blocking)
- Support common formats (PNG, JPEG, GIF, BMP, WebP)
- Handle errors gracefully (corrupt files, unsupported formats)
- Provide progress feedback for large files

**Function Signature:**
```rust
pub async fn load_image(path: &Path) -> Result<DynamicImage> {
    let bytes = tokio::fs::read(path).await?;
    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}
```

**Error Handling:**
- `IoError`: File not found, permission denied
- `ImageError`: Unsupported format, corrupt data
- Return `Result` with descriptive error messages

#### 3.3.2 `color_space.rs` - Color Conversion and Luminance

**Responsibilities:**
- Convert between color spaces (RGB, LAB, LCH, HSL)
- Compute perceptual luminance (CIE Y)
- Quantize colors to terminal palettes

**Key Functions:**
```rust
pub fn rgb_to_luminance(r: u8, g: u8, b: u8) -> f32 {
    // ITU-R BT.709 (HDTV) coefficients
    0.2126 * (r as f32 / 255.0) + 
    0.7152 * (g as f32 / 255.0) + 
    0.0722 * (b as f32 / 255.0)
}

pub fn quantize_to_ansi256(rgb: Rgb) -> u8 {
    // Find nearest color in 256-color palette
    // Use precomputed lookup table for speed
}
```

**Palette Handling:**
- Pre-generate ANSI 256-color palette (216 colors + 24 grayscale)
- Use KD-tree for fast nearest-color lookup (or precomputed LUT)

#### 3.3.3 `unicode_handler.rs` - Unicode Width and Validation

**Responsibilities:**
- Calculate display width of Unicode strings (East Asian Width)
- Validate grapheme clusters (no broken multi-byte sequences)
- Detect terminal Unicode support

**Key Functions:**
```rust
use unicode_width::UnicodeWidthStr;

pub fn display_width(s: &str) -> usize {
    s.width() // Handles East Asian Wide/Fullwidth characters
}

pub fn validate_graphemes(s: &str) -> bool {
    use unicode_segmentation::UnicodeSegmentation;
    // Ensure all grapheme clusters are well-formed
    s.graphemes(true).all(|g| g.len() > 0)
}
```

**Terminal Capability Detection:**
```rust
pub fn detect_unicode_support() -> UnicodeSupport {
    // Check LANG, LC_ALL environment variables
    // Check terminal emulator (TERM, TERM_PROGRAM)
    match std::env::var("LANG") {
        Ok(lang) if lang.contains("UTF-8") => UnicodeSupport::Full,
        _ => UnicodeSupport::Ascii,
    }
}
```

#### 3.3.4 `terminal_capabilities.rs` - Terminal Feature Detection

**Responsibilities:**
- Detect color support (no color, 16, 256, 24-bit RGB)
- Detect mouse support
- Detect terminal size and resize events
- Detect Unicode support

**Implementation:**
```rust
use crossterm::terminal;

pub struct TerminalCapabilities {
    pub color_support: ColorSupport,
    pub unicode_support: bool,
    pub mouse_support: bool,
    pub size: (u16, u16),
}

pub enum ColorSupport {
    NoColor,
    Color16,
    Color256,
    TrueColor,
}

pub fn detect_capabilities() -> TerminalCapabilities {
    let color = if terminal::supports_color() {
        detect_color_depth()
    } else {
        ColorSupport::NoColor
    };
    
    let (width, height) = terminal::size().unwrap_or((80, 24));
    
    TerminalCapabilities {
        color_support: color,
        unicode_support: detect_unicode_support(),
        mouse_support: true, // Crossterm always supports mouse
        size: (width, height),
    }
}
```

#### 3.3.5 `config.rs` - Configuration Management

**Responsibilities:**
- Load user preferences from config file (TOML)
- Save last-used settings per mode
- Provide sensible defaults

**Configuration Structure:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub ascii: AsciiPreferences,
    pub unicode: UnicodePreferences,
    pub text: TextPreferences,
    pub ui: UiPreferences,
}

#[derive(Serialize, Deserialize)]
pub struct AsciiPreferences {
    pub default_charset: CharacterSet,
    pub default_width: usize,
    pub edge_enhance: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ascii: AsciiPreferences {
                default_charset: CharacterSet::Extended,
                default_width: 80,
                edge_enhance: false,
            },
            // ... other defaults
        }
    }
}
```

**File Location:**
- Linux: `~/.config/terminal-art-studio/config.toml`
- macOS: `~/Library/Application Support/terminal-art-studio/config.toml`
- Windows: `%APPDATA%\terminal-art-studio\config.toml`

**Implementation:**
```rust
pub fn load_config() -> Result<Config> {
    let path = config_path()?;
    if path.exists() {
        let contents = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&contents)?)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path()?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    let contents = toml::to_string_pretty(config)?;
    std::fs::write(&path, contents)?;
    Ok(())
}
```

#### 3.3.6 `perf_monitor.rs` - Performance Monitoring

**Responsibilities:**
- Track frame times and FPS
- Track render times per engine
- Detect performance degradation
- Expose metrics to UI

**Implementation:**
```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub struct PerfMetrics {
    frame_times: VecDeque<Duration>, // Last 60 frames
    last_render_time: Duration,
    pub fps: f32,
    pub avg_frame_time_ms: f32,
    pub last_render_time_ms: u64,
}

impl PerfMetrics {
    pub fn new() -> Self {
        PerfMetrics {
            frame_times: VecDeque::with_capacity(60),
            last_render_time: Duration::default(),
            fps: 0.0,
            avg_frame_time_ms: 0.0,
            last_render_time_ms: 0,
        }
    }
    
    pub fn record_frame(&mut self, duration: Duration) {
        self.frame_times.push_back(duration);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
        
        let total: Duration = self.frame_times.iter().sum();
        let avg = total / self.frame_times.len() as u32;
        self.avg_frame_time_ms = avg.as_secs_f32() * 1000.0;
        self.fps = 1000.0 / self.avg_frame_time_ms;
    }
    
    pub fn record_render(&mut self, duration: Duration) {
        self.last_render_time = duration;
        self.last_render_time_ms = duration.as_millis() as u64;
    }
}
```

---

## 4. Threading and Performance Model

### 4.1 Threading Architecture

**Main Thread (UI Thread):**
- Runs the event loop (16ms target frame time for 60 FPS)
- Handles keyboard/mouse input
- Updates application state
- Renders UI with Ratatui
- Receives results from worker threads via channels

**Worker Threads (3 dedicated threads, 1 per rendering mode):**
- ASCII Render Worker: Processes image-to-ASCII requests
- Unicode Render Worker: Processes image-to-Unicode requests
- Text Stylizer Worker: Processes text stylization requests

**Async Runtime (Tokio, background tasks):**
- File I/O (loading images from disk)
- Configuration file I/O

### 4.2 Concurrency Model

**Message Passing (Actor Pattern):**
```rust
// Request messages (Main → Worker)
pub enum WorkerMessage {
    AsciiRequest { image: Arc<DynamicImage>, config: AsciiConfig },
    UnicodeRequest { image: Arc<DynamicImage>, config: UnicodeConfig },
    TextRequest { text: String, style: UnicodeStyle, gradient: GradientMode },
    Shutdown,
}

// Response messages (Worker → Main)
pub enum WorkerResponse {
    AsciiComplete(String),
    UnicodeComplete(String),
    TextComplete(String),
    Error(String),
}
```

**Channel Types:**
- Main → Worker: `mpsc::Sender<WorkerMessage>` (multiple producers, single consumer per worker)
- Worker → Main: `mpsc::Sender<WorkerResponse>` (multiple producers, single consumer on main)

**Arc Usage (Zero-Copy Image Sharing):**
- Images are wrapped in `Arc<DynamicImage>` to avoid cloning large buffers
- Workers receive `Arc` clones (reference count increment, no data copy)
- Main thread retains `Arc` for potential re-renders with different settings

### 4.3 Performance Optimizations

**1. Allocation Minimization:**
- Pre-allocate output string buffers with known capacity
- Reuse buffers where possible (via object pools or `Vec::with_capacity`)
- Avoid allocations in hot loops (character mapping, color quantization)

**2. Caching:**
- Cache decoded images (keyed by file path + modification time)
- Cache character set luminance lookup tables
- Cache terminal color palette (quantization LUT)

**3. Adaptive Scaling:**
- Detect terminal size changes
- Automatically adjust output dimensions
- Warn user if output exceeds terminal size (with option to override)

**4. Debouncing:**
- Debounce rapid setting changes (e.g., user holding +/- key)
- Batch render requests: only send new request after 100ms of inactivity
- Cancel in-flight renders when new request arrives

**5. SIMD and Vectorization:**
- Leverage `image` crate's SIMD optimizations for resizing and color conversion
- Use `rayon` for data-parallel operations (e.g., per-row processing) if needed

**6. Memory Layout:**
- Use contiguous buffers (Vec) over linked structures
- Prefer stack allocation for small structures
- Use `Cow<str>` for strings that may or may not need cloning

### 4.4 Frame Time Budget (60 FPS → 16ms per frame)

**Breakdown:**
- Input handling: < 1ms
- State updates: < 1ms
- Ratatui rendering: < 5ms (depends on terminal size and complexity)
- Worker communication (non-blocking): < 0.1ms
- Buffer: 9ms for headroom

**Render Time Budget (Off-Thread):**
- ASCII rendering: < 50ms for 200x100 output
- Unicode rendering: < 100ms for 200x100 output (color quantization overhead)
- Text stylization: < 10ms for 1000 characters

**Enforcement:**
- Use `std::time::Instant` to measure actual times
- Log warnings if frame times exceed 16ms
- Display metrics in status bar for user visibility

---

## 5. Data Flow and Rendering Pipeline

### 5.1 Image-to-ASCII/Unicode Pipeline

```
┌────────────┐
│ User Input │ (Press 'L' to load image)
└─────┬──────┘
      │
      v
┌─────────────────┐
│ File Dialog     │ (async, via tokio)
│ (or fzf)        │
└─────┬───────────┘
      │
      v
┌─────────────────────┐
│ Image Loader        │ (async, decode in background)
│ image::load_from_   │
│   memory()          │
└─────┬───────────────┘
      │
      v
┌─────────────────────┐
│ Store in AppState   │ (Arc<DynamicImage>)
│ state.input_image = │
│   Some(Arc(img))    │
└─────┬───────────────┘
      │
      v
┌─────────────────────┐
│ User Adjusts        │ (Change settings via control panel)
│ Settings            │ (Width, charset, color mode, etc.)
└─────┬───────────────┘
      │
      v (Debounced, after 100ms of inactivity)
┌─────────────────────────────┐
│ Send to Worker Thread       │
│ worker_tx.send(             │
│   WorkerMessage::           │
│     AsciiRequest { ... }    │
│ )                           │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐ (Worker Thread)
│ Render Engine               │
│ - Resize image              │
│ - Convert to grayscale      │
│ - Map luminance → chars     │
│ - Format as string          │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐
│ Send Result Back            │
│ main_tx.send(               │
│   WorkerResponse::          │
│     AsciiComplete(output)   │
│ )                           │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐ (Main Thread)
│ Update AppState             │
│ state.preview_content =     │
│   Some(output)              │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐
│ Render UI                   │ (Next frame)
│ Display preview_content     │
│ in preview widget           │
└─────────────────────────────┘
```

### 5.2 Text Stylization Pipeline

```
┌────────────┐
│ User Input │ (Type text in control panel)
└─────┬──────┘
      │
      v
┌─────────────────┐
│ Store in State  │ (state.text_state.input_text)
└─────┬───────────┘
      │
      v (On Enter or setting change)
┌─────────────────────────────┐
│ Send to Worker Thread       │
│ worker_tx.send(             │
│   WorkerMessage::           │
│     TextRequest { ... }     │
│ )                           │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐ (Worker Thread)
│ Text Stylizer Engine        │
│ - Apply Unicode style       │
│ - Apply gradient colors     │
│ - Validate graphemes        │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐
│ Send Result Back            │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐ (Main Thread)
│ Update AppState             │
│ state.preview_content =     │
│   Some(styled_text)         │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐
│ Render UI                   │
└─────────────────────────────┘
```

### 5.3 Error Handling Flow

```
┌─────────────────────┐
│ Error Occurs        │ (File not found, decode error, etc.)
└─────┬───────────────┘
      │
      v
┌─────────────────────────────┐
│ Worker Sends Error          │
│ main_tx.send(               │
│   WorkerResponse::Error(e)  │
│ )                           │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐ (Main Thread)
│ Update Status Message       │
│ state.status_message =      │
│   format!("Error: {}", e)   │
└─────┬───────────────────────┘
      │
      v
┌─────────────────────────────┐
│ Display in Status Bar       │ (Red text, persists for 5 seconds)
└─────────────────────────────┘
```

---

## 6. UI/UX Layout and Navigation Design

### 6.1 Layout Specifications

**Title Bar (1 line):**
- Left: Application name and version
- Right: Quick shortcuts (`[?] Help`, `[Q] Quit`)

**Main Content Area (min 10 lines):**
- **Left Column (20-30 cols, adaptive):**
  - Mode Selector (3-5 lines)
  - Control Panel (remaining space)
  
- **Right Column (remaining space):**
  - Preview Area (scrollable if output exceeds terminal size)

**Status Bar (1 line):**
- Status message (left)
- Performance metrics (right): `FPS: 60 | Render: 12ms | File: image.png`

### 6.2 Mode Selector Widget

**Visual Representation:**
```
┌──────────────┐
│ Mode:        │
│ [•] ASCII    │  <- Selected (filled bullet)
│ [ ] Unicode  │
│ [ ] Text     │
└──────────────┘
```

**Navigation:**
- Arrow Up/Down: Move selection
- `1`, `2`, `3`: Jump to specific mode
- Enter: Confirm selection (switches mode)

**Behavior:**
- Switching modes preserves last-used settings for each mode
- Preview is cleared until new render completes
- Status message shows "Switched to [Mode] mode"

### 6.3 Control Panel Widget

**ASCII Mode Controls:**
```
┌──────────────────────┐
│ Settings:            │
│                      │
│ Width: 80            │ <- Numeric input (+/- keys)
│ Charset: Extended    │ <- Selection (left/right arrows)
│ Edge Enhance: Off    │ <- Toggle (Space)
│ Invert: Off          │ <- Toggle (Space)
│                      │
│ [Space] Render       │ <- Action button
│ [L] Load Image       │
│ [S] Save Output      │
└──────────────────────┘
```

**Unicode Mode Controls:**
```
┌──────────────────────┐
│ Settings:            │
│                      │
│ Width: 80            │
│ Mode: Half-Blocks    │ <- Selection
│ Color: 256-color     │ <- Selection
│                      │
│ [Space] Render       │
│ [L] Load Image       │
│ [S] Save Output      │
└──────────────────────┘
```

**Text Stylizer Mode Controls:**
```
┌──────────────────────┐
│ Settings:            │
│                      │
│ Style: Bold          │ <- Selection
│ Gradient: Horizontal │ <- Selection
│ Start Color: #FF0000 │ <- Color picker (simplified)
│ End Color: #0000FF   │
│                      │
│ Input Text:          │
│ [Type here...]       │ <- Text input field
│                      │
│ [Enter] Stylize      │
│ [S] Save Output      │
└──────────────────────┘
```

**Navigation Within Control Panel:**
- Arrow Up/Down: Move between settings
- Arrow Left/Right: Adjust selection-based settings
- `+/-`: Adjust numeric settings
- Space: Toggle boolean settings
- Enter: Trigger action (Render, Stylize)
- Type directly for text input fields

### 6.4 Preview Area Widget

**Features:**
- Scrollable (if output exceeds visible area)
- Syntax highlighting for ANSI color codes (rendered as actual colors)
- Line numbers (optional, toggled with `N` key)
- Word wrap (optional, toggled with `W` key)

**Scrolling:**
- Arrow Up/Down: Scroll by line
- Page Up/Down: Scroll by page
- Home/End: Jump to top/bottom
- Mouse wheel: Scroll by line (if mouse enabled)

**Actions:**
- `S`: Save output to file
- `C`: Copy output to clipboard (via external command: `xclip`, `pbcopy`, `clip.exe`)
- `E`: Export to file with metadata (timestamp, settings used)

### 6.5 Help Overlay

**Triggered by `?` key, dismissed by `Esc` or `?`**

**Layout:**
```
┌─────────────────────────────────────────────────────────────────┐
│                         Keyboard Shortcuts                       │
├─────────────────────────────────────────────────────────────────┤
│ Global:                                                          │
│   Q         Quit application                                     │
│   ?         Toggle help overlay                                  │
│   Tab       Next widget                                          │
│   Shift+Tab Previous widget                                      │
│   Esc       Cancel / Close overlay                               │
│                                                                  │
│ Mode Selector:                                                   │
│   1, 2, 3   Jump to mode                                         │
│   ↑ ↓       Navigate modes                                       │
│   Enter     Select mode                                          │
│                                                                  │
│ Control Panel:                                                   │
│   ↑ ↓       Navigate settings                                    │
│   ← →       Adjust selection                                     │
│   + -       Adjust numeric values                                │
│   Space     Toggle boolean / Trigger render                      │
│   L         Load image                                           │
│   S         Save output                                          │
│                                                                  │
│ Preview Area:                                                    │
│   ↑ ↓       Scroll by line                                       │
│   PgUp PgDn Scroll by page                                       │
│   Home End  Jump to top/bottom                                   │
│   S         Save output                                          │
│   C         Copy to clipboard                                    │
│   N         Toggle line numbers                                  │
│   W         Toggle word wrap                                     │
│                                                                  │
│                    [Press ? or Esc to close]                     │
└─────────────────────────────────────────────────────────────────┘
```

**Styling:**
- Semi-transparent background (dims main UI)
- Centered overlay (60% of terminal width, adaptive height)
- Border with rounded corners (if terminal supports)

### 6.6 Accessibility Considerations

**Keyboard-Only Navigation:**
- All features accessible without mouse
- Logical tab order
- Clear focus indicators (highlighted borders)

**Visual Clarity:**
- High contrast colors for text and borders
- Consistent color scheme (configurable in future)
- No reliance on color alone for information (use symbols too)

**Discoverable Controls:**
- Inline hints in control panel (`[Space] Render`)
- Always-visible help key hint in title bar (`[?] Help`)
- Status bar shows context-sensitive shortcuts

---

## 7. Testing Strategy

### 7.1 Unit Tests

**Scope:** Test individual functions and modules in isolation

**Coverage:**
- **Render Engines:** Test character mapping, luminance calculation, color quantization
- **Unicode Handling:** Test width calculation, grapheme validation
- **Color Space Conversion:** Test RGB to LAB, luminance calculation
- **State Management:** Test state transitions, setting persistence

**Example Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luminance_to_char_mapping() {
        let charset = CharacterSet::Standard;
        assert_eq!(map_luminance_to_char(0.0, &charset), ' ');
        assert_eq!(map_luminance_to_char(1.0, &charset), '@');
    }

    #[test]
    fn test_unicode_width_calculation() {
        assert_eq!(display_width("Hello"), 5);
        assert_eq!(display_width("你好"), 4); // Chinese characters are 2 cells wide
        assert_eq!(display_width("🎨"), 2);   // Emoji is 2 cells wide
    }

    #[test]
    fn test_color_quantization() {
        let rgb = Rgb::new(128, 128, 128);
        let ansi = quantize_to_ansi256(rgb);
        assert!(ansi >= 232 && ansi <= 255); // Grayscale range
    }
}
```

**Test Execution:**
```bash
cargo test
cargo test --release  # Test with optimizations enabled
```

### 7.2 Integration Tests

**Scope:** Test module interactions and end-to-end workflows

**Coverage:**
- **Full Rendering Pipeline:** Load image → Render → Verify output format
- **Worker Communication:** Send request → Receive response → Verify correctness
- **Error Handling:** Trigger errors → Verify graceful degradation

**Example Tests:**
```rust
#[test]
fn test_ascii_rendering_pipeline() {
    let image = load_test_image("test_images/sample.png").unwrap();
    let config = AsciiConfig {
        target_width: 80,
        charset: CharacterSet::Extended,
        edge_enhance: false,
        invert: false,
    };
    let output = render_ascii(&image, &config).unwrap();
    
    // Verify output dimensions
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0].chars().count(), 80);
    
    // Verify no empty lines
    assert!(lines.iter().all(|line| !line.is_empty()));
}

#[test]
fn test_worker_communication() {
    let (tx, rx) = mpsc::channel();
    let worker_tx = tx.clone();
    
    std::thread::spawn(move || {
        ascii_worker(rx, worker_tx);
    });
    
    let image = Arc::new(load_test_image("test.png").unwrap());
    tx.send(WorkerMessage::AsciiRequest {
        image: image.clone(),
        config: AsciiConfig::default(),
    }).unwrap();
    
    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(WorkerResponse::AsciiComplete(output)) => {
            assert!(!output.is_empty());
        }
        _ => panic!("Worker did not respond"),
    }
}
```

### 7.3 Snapshot Tests

**Scope:** Verify output consistency across code changes

**Coverage:**
- **Rendering Output:** Snapshot ASCII/Unicode art for reference images
- **UI Layout:** Snapshot terminal UI (via `insta` crate)

**Implementation:**
```rust
use insta::assert_snapshot;

#[test]
fn test_ascii_output_snapshot() {
    let image = load_test_image("reference.png").unwrap();
    let config = AsciiConfig::default();
    let output = render_ascii(&image, &config).unwrap();
    
    assert_snapshot!(output);
}
```

**Update Snapshots:**
```bash
cargo insta review  # Review and accept snapshot changes
```

### 7.4 Property-Based Tests

**Scope:** Test invariants and edge cases automatically

**Coverage:**
- **Unicode Mapping:** Every ASCII char maps to valid Unicode char
- **Luminance Range:** Luminance values always in [0, 1]
- **Display Width:** Sum of character widths matches expected total

**Implementation:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_luminance_in_range(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255) {
        let luminance = rgb_to_luminance(r, g, b);
        assert!(luminance >= 0.0 && luminance <= 1.0);
    }

    #[test]
    fn test_unicode_style_preserves_length(text in "\\PC*") {
        let styled = apply_unicode_style(&text, UnicodeStyle::Bold).unwrap();
        assert_eq!(text.chars().count(), styled.chars().count());
    }
}
```

### 7.5 Benchmarking

**Scope:** Measure performance of critical paths

**Coverage:**
- **Render Engines:** Time per frame for various image sizes
- **Color Quantization:** Lookup table performance
- **Unicode Operations:** Width calculation, grapheme segmentation

**Implementation:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_ascii_render(c: &mut Criterion) {
    let image = load_test_image("benchmark.png").unwrap();
    let config = AsciiConfig::default();
    
    c.bench_function("ascii_render_80x40", |b| {
        b.iter(|| render_ascii(black_box(&image), black_box(&config)))
    });
}

criterion_group!(benches, benchmark_ascii_render);
criterion_main!(benches);
```

**Execution:**
```bash
cargo bench
cargo flamegraph --bench ascii_render  # Generate flamegraph for profiling
```

### 7.6 Manual Testing Checklist

**Cross-Platform:**
- [ ] Test on Linux (Gnome Terminal, Alacritty, Kitty)
- [ ] Test on macOS (Terminal.app, iTerm2)
- [ ] Test on Windows (Windows Terminal, PowerShell)

**Functional:**
- [ ] Load various image formats (PNG, JPEG, GIF, WebP)
- [ ] Test with large images (>10MB)
- [ ] Test with corrupt/invalid files
- [ ] Test all rendering modes
- [ ] Test all settings combinations
- [ ] Test terminal resize during rendering
- [ ] Test rapid setting changes (debouncing)
- [ ] Test save/load config persistence

**Performance:**
- [ ] Verify 60 FPS in idle state
- [ ] Verify no UI blocking during rendering
- [ ] Verify render times < 100ms for 200x100 output
- [ ] Verify memory usage stays stable (no leaks)

**UI/UX:**
- [ ] Test keyboard navigation (all widgets)
- [ ] Test help overlay
- [ ] Test focus indicators
- [ ] Test status messages
- [ ] Test error handling (user-visible errors)

---

## 8. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Goals:**
- Set up project structure
- Implement basic TUI shell
- Establish threading model
- Implement terminal capability detection

**Deliverables:**
- `main.rs` with event loop and panic hook
- `state.rs` with initial state structure
- `ui/mod.rs` with basic layout (title bar, status bar, placeholder widgets)
- `terminal_capabilities.rs` with color and Unicode detection
- `config.rs` with default configuration

**Acceptance Criteria:**
- Application launches, displays empty UI, responds to keyboard input (Q to quit)
- Terminal is restored on exit (no artifacts left behind)
- Status bar displays terminal size and capabilities

### Phase 2: Image-to-ASCII Engine (Weeks 3-4)

**Goals:**
- Implement ASCII rendering engine
- Implement worker thread communication
- Implement image loading (async)

**Deliverables:**
- `render_engines/ascii.rs` with complete implementation
- `image_loader.rs` with async file loading
- `color_space.rs` with luminance calculation
- Worker thread setup in `main.rs`
- Control panel widget for ASCII mode

**Acceptance Criteria:**
- Can load PNG/JPEG images
- Can render ASCII art with configurable width and charset
- Rendering happens on worker thread (UI never blocks)
- Results display in preview area
- Performance: <50ms for 200x100 output

### Phase 3: UI Polish and Navigation (Week 5)

**Goals:**
- Implement mode selector widget
- Implement preview area scrolling
- Implement help overlay
- Implement keyboard shortcuts

**Deliverables:**
- `ui/mode_selector.rs` with navigation logic
- `ui/preview.rs` with scrolling and line numbers
- `ui/help.rs` with overlay rendering
- `input.rs` with complete key bindings

**Acceptance Criteria:**
- Tab navigation works across all widgets
- Help overlay shows all shortcuts
- Preview area scrolls smoothly
- All shortcuts work as specified

### Phase 4: Image-to-Unicode Engine (Week 6)

**Goals:**
- Implement Unicode rendering engine
- Implement color support (ANSI 256 and 24-bit RGB)
- Implement half-block optimization

**Deliverables:**
- `render_engines/unicode.rs` with all modes (Blocks, HalfBlocks, Braille)
- `color_space.rs` extended with color quantization
- Control panel widget for Unicode mode

**Acceptance Criteria:**
- Can render Unicode art with color
- Half-block mode produces 2x vertical resolution
- Color quantization works correctly (verified by snapshots)
- Performance: <100ms for 200x100 output

### Phase 5: Text Stylizer Engine (Week 7)

**Goals:**
- Implement Unicode text stylization
- Implement gradient coloring
- Implement grapheme cluster validation

**Deliverables:**
- `render_engines/text_stylizer.rs` with all Unicode styles
- `unicode_handler.rs` with width and validation functions
- Control panel widget for text mode with input field

**Acceptance Criteria:**
- Can stylize text with all Unicode styles
- Gradients work correctly (horizontal, vertical, per-character)
- No broken grapheme clusters in output
- Performance: <10ms for 1000 characters

### Phase 6: File I/O and Configuration (Week 8)

**Goals:**
- Implement save output functionality
- Implement configuration persistence
- Implement copy to clipboard

**Deliverables:**
- Save output to file (with metadata)
- Load/save config on startup/shutdown
- Clipboard integration (platform-specific)

**Acceptance Criteria:**
- Can save ASCII/Unicode art to text file
- Settings persist across application restarts
- Clipboard copy works on Linux, macOS, Windows

### Phase 7: Testing and Benchmarking (Weeks 9-10)

**Goals:**
- Write comprehensive unit tests
- Write integration tests
- Create snapshot tests
- Run benchmarks and optimize

**Deliverables:**
- Test coverage >80% for critical modules
- Benchmarks for all rendering engines
- Performance profiling results (flamegraphs)

**Acceptance Criteria:**
- All tests pass
- No memory leaks (verified with `valgrind` or similar)
- Rendering times meet targets (<50ms ASCII, <100ms Unicode)
- FPS >60 in idle state

### Phase 8: Distribution and Documentation (Weeks 11-12)

**Goals:**
- Set up CI/CD pipeline
- Create release builds for all platforms
- Write user documentation
- Create demo videos/screenshots

**Deliverables:**
- GitHub Actions workflow for CI (test, lint, build)
- Release binaries for Linux (x86_64, ARM64), macOS (Intel, ARM), Windows (x86_64)
- README with installation instructions and screenshots
- User guide (in-app help is already implemented)

**Acceptance Criteria:**
- Single-binary distribution works on all platforms
- No runtime dependencies required
- Installation instructions are clear and tested
- GitHub releases are automated

---

## 9. Tooling and Dependencies

### 9.1 Core Dependencies (Cargo.toml)

```toml
[package]
name = "terminal-art-studio"
version = "1.0.0"
edition = "2021"
rust-version = "1.75.0"  # Minimum supported Rust version

[dependencies]
# TUI Framework
ratatui = "0.28"
crossterm = "0.27"

# Image Processing
image = { version = "0.25", default-features = false, features = ["png", "jpeg", "gif", "webp"] }
imageproc = "0.25"  # For advanced image operations (edge detection)

# Color Handling
palette = "0.7"

# Unicode Support
unicode-width = "0.1"
unicode-segmentation = "1.10"

# Concurrency
tokio = { version = "1.35", features = ["rt-multi-thread", "fs", "io-util"] }
crossbeam-channel = "0.5"  # Alternative to std::mpsc (faster, more features)

# Configuration
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# Error Handling
anyhow = "1.0"
thiserror = "1.0"

# CLI (for file picker integration)
dialoguer = "0.11"  # For interactive prompts if needed

[dev-dependencies]
# Testing
criterion = "0.5"  # Benchmarking
proptest = "1.4"   # Property-based testing
insta = "1.34"     # Snapshot testing

# Profiling
flamegraph = "0.6"

[profile.release]
opt-level = 3
lto = "fat"        # Link-time optimization for smaller binaries
codegen-units = 1  # Better optimization at cost of compile time
strip = true       # Strip symbols for smaller binary size
```

### 9.2 Development Tools

**Linting and Formatting:**
```bash
rustfmt --edition 2021 src/**/*.rs  # Format code
cargo clippy -- -D warnings         # Lint with strict mode
```

**Testing:**
```bash
cargo test                          # Run all tests
cargo test --release                # Test with optimizations
cargo bench                         # Run benchmarks
cargo insta test                    # Run snapshot tests
cargo insta review                  # Review snapshot changes
```

**Profiling:**
```bash
cargo flamegraph --bench ascii_render  # CPU profiling
valgrind --leak-check=full ./target/release/terminal-art-studio  # Memory leak detection
heaptrack ./target/release/terminal-art-studio  # Heap profiling
```

**Cross-Compilation:**
```bash
# Linux to Windows
cargo build --release --target x86_64-pc-windows-gnu

# Linux to macOS (requires cross-compilation toolchain)
cargo build --release --target x86_64-apple-darwin

# For ARM targets
cargo build --release --target aarch64-unknown-linux-gnu
```

### 9.3 CI/CD Pipeline (GitHub Actions)

**Workflow: `.github/workflows/ci.yml`**
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings

  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v3
        with:
          name: terminal-art-studio-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/terminal-art-studio*

  release:
    needs: [test, build]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/')
    steps:
      - uses: actions/download-artifact@v3
      - uses: softprops/action-gh-release@v1
        with:
          files: |
            terminal-art-studio-*/terminal-art-studio*
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 9.4 Build Scripts

**Linux/macOS Build:**
```bash
#!/bin/bash
# build.sh

set -e

echo "Building Terminal Art Studio..."

# Format and lint
cargo fmt --check
cargo clippy -- -D warnings

# Test
cargo test --release

# Build release binary
cargo build --release

# Strip binary (if not already done by profile)
strip target/release/terminal-art-studio

echo "Build complete: target/release/terminal-art-studio"
echo "Binary size: $(du -h target/release/terminal-art-studio | cut -f1)"
```

**Windows Build (PowerShell):**
```powershell
# build.ps1

Write-Host "Building Terminal Art Studio..."

# Format and lint
cargo fmt --check
cargo clippy -- -D warnings

# Test
cargo test --release

# Build release binary
cargo build --release

Write-Host "Build complete: target\release\terminal-art-studio.exe"
$size = (Get-Item target\release\terminal-art-studio.exe).Length / 1MB
Write-Host "Binary size: $size MB"
```

---

## 10. Risks, Trade-offs, and Mitigations

### 10.1 Performance Risks

**Risk: Rendering large images may exceed frame time budget**
- **Likelihood:** Medium
- **Impact:** High (UI lag, poor user experience)
- **Mitigation:**
  - Implement adaptive downsampling (warn user if image is too large)
  - Add progress indicator for renders >200ms
  - Allow user to cancel in-flight renders
  - Benchmark with various image sizes during testing

**Risk: Terminal size changes during rendering may cause artifacts**
- **Likelihood:** Low
- **Impact:** Medium (visual glitches, incorrect layout)
- **Mitigation:**
  - Detect resize events immediately
  - Cancel in-flight renders on resize
  - Re-render with new dimensions
  - Add debouncing to avoid rapid re-renders

**Risk: ANSI color codes may not render correctly on all terminals**
- **Likelihood:** Medium
- **Impact:** Low (degraded output, but still functional)
- **Mitigation:**
  - Detect terminal color support at startup
  - Fallback to grayscale if color not supported
  - Provide user option to override auto-detection
  - Test on multiple terminal emulators

### 10.2 Cross-Platform Risks

**Risk: Windows terminal may have limited Unicode support**
- **Likelihood:** Medium (older Windows versions, legacy console)
- **Impact:** Medium (degraded output)
- **Mitigation:**
  - Detect Unicode support explicitly
  - Provide ASCII fallback mode
  - Recommend Windows Terminal (modern, full Unicode support)
  - Test on Windows 10/11 with both legacy console and Windows Terminal

**Risk: macOS terminal may have different character width calculations**
- **Likelihood:** Low (macOS uses standard East Asian Width tables)
- **Impact:** Low (minor alignment issues)
- **Mitigation:**
  - Use `unicode-width` crate (standards-compliant)
  - Verify rendering on both Terminal.app and iTerm2
  - Include snapshot tests for alignment

**Risk: File path handling differs across platforms**
- **Likelihood:** Low (Rust's `std::path` handles this)
- **Impact:** Low (file not found errors)
- **Mitigation:**
  - Use `std::path::PathBuf` everywhere
  - Test with paths containing spaces, Unicode, special characters
  - Use platform-specific file dialogs (via `dialoguer` or native)

### 10.3 Usability Risks

**Risk: Keyboard shortcuts may conflict with terminal emulator shortcuts**
- **Likelihood:** Medium
- **Impact:** Medium (features inaccessible)
- **Mitigation:**
  - Avoid common conflicts (Ctrl+C, Ctrl+Z, Ctrl+D)
  - Use alternative shortcuts (e.g., `Q` instead of Ctrl+Q)
  - Document conflicts in help overlay
  - Provide configurable key bindings (future enhancement)

**Risk: Users may not understand Unicode text styles (unfamiliar with mathematical alphanumeric symbols)**
- **Likelihood:** High
- **Impact:** Low (confusion, but discoverable via UI)
- **Mitigation:**
  - Provide live preview (instant feedback)
  - Include examples in help overlay
  - Add descriptive names (e.g., "Bold" instead of "U+1D400")

**Risk: Clipboard integration may fail on some systems**
- **Likelihood:** Medium (requires external commands: xclip, pbcopy, clip.exe)
- **Impact:** Low (feature unavailable, but save-to-file works)
- **Mitigation:**
  - Detect clipboard command availability at startup
  - Disable feature if not available (show message)
  - Provide fallback: save to temporary file and show path
  - Document clipboard dependencies in README

### 10.4 Dependency Risks

**Risk: Ratatui API changes may break compatibility**
- **Likelihood:** Low (stable API post-v0.20)
- **Impact:** High (requires code rewrite)
- **Mitigation:**
  - Pin major version in Cargo.toml
  - Review changelogs before upgrading
  - Maintain compatibility with N and N-1 minor versions

**Risk: Image crate may not support some formats**
- **Likelihood:** Medium (e.g., AVIF, HEIC)
- **Impact:** Low (user can convert externally)
- **Mitigation:**
  - Clearly document supported formats
  - Provide helpful error messages ("Format not supported: AVIF. Try converting to PNG.")
  - Consider adding support via feature flags in future

**Risk: Tokio overhead may impact startup time**
- **Likelihood:** Low
- **Impact:** Low (slightly slower startup)
- **Mitigation:**
  - Use minimal Tokio features (only `rt-multi-thread` and `fs`)
  - Measure startup time during benchmarking
  - Consider alternative async runtime if overhead is significant (unlikely)

### 10.5 Maintenance Risks

**Risk: Code complexity may increase over time, reducing maintainability**
- **Likelihood:** High (all software projects)
- **Impact:** High (slower development, more bugs)
- **Mitigation:**
  - Enforce strict module boundaries (no circular dependencies)
  - Write comprehensive documentation (doc comments)
  - Maintain high test coverage (>80%)
  - Conduct code reviews (even for solo projects, review before merging)

**Risk: Performance regressions may go unnoticed**
- **Likelihood:** Medium
- **Impact:** Medium (gradual degradation)
- **Mitigation:**
  - Run benchmarks on every release
  - Track performance metrics over time (store results in repo)
  - Add benchmark CI job (fail if >10% regression)

---

## 11. Future Enhancements (Post-MVP)

### 11.1 Additional Rendering Modes

**Video-to-ASCII:**
- Use `ffmpeg` library bindings to decode video frames
- Render each frame to ASCII, output as animated sequence
- Support output to video file (re-encode with ffmpeg)

**Webcam-to-ASCII:**
- Use `opencv` library bindings to capture webcam frames
- Real-time ASCII rendering (target 30 FPS)
- Record to file (optional)

### 11.2 Advanced Features

**Custom Character Sets:**
- Allow users to define custom character sets (via config file)
- Provide UI for testing and previewing custom sets

**Dithering Algorithms:**
- Floyd-Steinberg dithering for ASCII art
- Ordered dithering for smoother gradients

**ASCII Art Filters:**
- Sobel edge detection (already planned for edge enhancement)
- Additional filters: sharpen, blur, emboss

**Batch Processing:**
- Process multiple images at once (CLI mode)
- Output to directory with consistent naming

### 11.3 UI Improvements

**Themes:**
- Dark mode / light mode
- Configurable color schemes (via config file)

**Mouse Support:**
- Click to select widgets
- Drag to adjust numeric sliders
- Scroll preview area with mouse wheel

**Windowing:**
- Multiple preview windows (side-by-side comparison)
- Split panes (settings on left, multiple previews on right)

### 11.4 Export Formats

**HTML Export:**
- Embed ASCII/Unicode art in HTML with CSS styling
- Preserve ANSI colors as inline styles

**SVG Export:**
- Convert ASCII/Unicode art to vector format
- Scalable, high-quality output

**Image Export:**
- Render ASCII/Unicode art as raster image (PNG)
- Use monospace font, configurable size

### 11.5 Plugin System

**User Scripts:**
- Allow users to write custom rendering algorithms in Lua or JavaScript
- Expose safe API for image processing and character mapping

**Filter Plugins:**
- Load external filters (written in Rust, compiled as dynamic libraries)
- Provide plugin discovery and management UI

---

## 12. Conclusion

This technical plan provides a comprehensive, implementation-ready blueprint for a high-performance Terminal User Interface art rendering studio. The architecture is designed with performance, maintainability, and extensibility as core principles.

**Key Strengths of This Approach:**
1. **Proven Technology Stack:** Rust + Ratatui + Crossterm is battle-tested and widely adopted
2. **Clear Separation of Concerns:** UI, state, and rendering are strictly decoupled
3. **Performance-First Design:** Worker threads, zero-copy where possible, adaptive scaling
4. **Rigorous Testing Strategy:** Unit, integration, snapshot, and property-based tests
5. **Cross-Platform from Day One:** No platform-specific code in core logic
6. **Extensible Architecture:** New rendering modes can be added without UI rewrites

**Expected Outcomes:**
- **Performance:** 60 FPS UI with <100ms render times for typical inputs
- **Quality:** Professional-grade output with correct Unicode handling and color accuracy
- **Usability:** Keyboard-first navigation with discoverable controls
- **Maintainability:** Clean architecture with >80% test coverage

**Timeline:** 8-12 weeks from start to production-ready MVP, with 2-4 additional weeks for distribution and documentation.

By following this plan precisely, a skilled engineer will be able to build a portfolio-grade TUI application that showcases both technical excellence and user-centric design.

---

**Document End**
