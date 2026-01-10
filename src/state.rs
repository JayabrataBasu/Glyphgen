//! Application state management
//!
//! Single source of truth for application state with mode-specific substates.

use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::Sender;
use image::DynamicImage;

use crate::config::Config;
use crate::perf_monitor::PerfMetrics;
use crate::render_engines::{
    ascii::CharacterSet, text_stylizer::GradientMode, text_stylizer::UnicodeStyle,
    unicode::UnicodeMode,
};
use crate::terminal_capabilities::{ColorSupport, TerminalCapabilities};
use crate::worker::{WorkerMessage, WorkerResponse};

/// Main render mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    #[default]
    ImageToAscii,
    ImageToUnicode,
    TextStylizer,
}

impl RenderMode {
    pub fn name(&self) -> &'static str {
        match self {
            RenderMode::ImageToAscii => "ASCII Art",
            RenderMode::ImageToUnicode => "Unicode Art",
            RenderMode::TextStylizer => "Text Stylizer",
        }
    }

    pub fn all() -> &'static [RenderMode] {
        &ALL_RENDER_MODES[..]
    }
}

// Module-level static array used by `RenderMode::all()` to ensure a &'static slice
static ALL_RENDER_MODES: [RenderMode; 3] = [
    RenderMode::ImageToAscii,
    RenderMode::ImageToUnicode,
    RenderMode::TextStylizer,
];

/// Which widget is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedWidget {
    #[default]
    ModeSelector,
    ControlPanel,
    Preview,
}

impl FocusedWidget {
    pub fn next(&self) -> Self {
        match self {
            FocusedWidget::ModeSelector => FocusedWidget::ControlPanel,
            FocusedWidget::ControlPanel => FocusedWidget::Preview,
            FocusedWidget::Preview => FocusedWidget::ModeSelector,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            FocusedWidget::ModeSelector => FocusedWidget::Preview,
            FocusedWidget::ControlPanel => FocusedWidget::ModeSelector,
            FocusedWidget::Preview => FocusedWidget::ControlPanel,
        }
    }
}

/// ASCII rendering state
#[derive(Debug, Clone)]
pub struct AsciiRenderState {
    pub charset: CharacterSet,
    pub width: usize,
    pub invert: bool,
    pub edge_enhance: bool,
    pub selected_setting: usize,
}

impl Default for AsciiRenderState {
    fn default() -> Self {
        Self {
            charset: CharacterSet::Extended,
            width: 80,
            invert: false,
            edge_enhance: false,
            selected_setting: 0,
        }
    }
}

impl AsciiRenderState {
    pub fn settings_count() -> usize {
        5 // width, charset, invert, edge_enhance, output_format
    }

    pub fn setting_name(&self, index: usize) -> &'static str {
        match index {
            0 => "Width",
            1 => "Charset",
            2 => "Invert",
            3 => "Edge Enhance",
            _ => "Unknown",
        }
    }

    pub fn setting_value(&self, index: usize) -> String {
        match index {
            0 => format!("{}", self.width),
            1 => self.charset.name().to_string(),
            2 => if self.invert { "On" } else { "Off" }.to_string(),
            3 => if self.edge_enhance { "On" } else { "Off" }.to_string(),
            _ => String::new(),
        }
    }
}

/// Unicode rendering state
#[derive(Debug, Clone)]
pub struct UnicodeRenderState {
    pub mode: UnicodeMode,
    pub width: usize,
    pub color_mode: ColorSupport,
    pub selected_setting: usize,
}

impl Default for UnicodeRenderState {
    fn default() -> Self {
        Self {
            mode: UnicodeMode::HalfBlocks,
            width: 80,
            color_mode: ColorSupport::TrueColor,
            selected_setting: 0,
        }
    }
}

impl UnicodeRenderState {
    pub fn settings_count() -> usize {
        4 // width, mode, color, output_format
    }

    pub fn setting_name(&self, index: usize) -> &'static str {
        match index {
            0 => "Width",
            1 => "Mode",
            2 => "Color",
            _ => "Unknown",
        }
    }

    pub fn setting_value(&self, index: usize) -> String {
        match index {
            0 => format!("{}", self.width),
            1 => self.mode.name().to_string(),
            2 => self.color_mode.name().to_string(),
            _ => String::new(),
        }
    }
}

/// Text stylizer state
#[derive(Debug, Clone)]
pub struct TextStylizeState {
    pub style: UnicodeStyle,
    pub gradient: GradientMode,
    pub start_color: (u8, u8, u8),
    pub end_color: (u8, u8, u8),
    pub input_text: String,
    pub cursor_position: usize,
    pub selected_setting: usize,
    pub editing_text: bool,
}

impl Default for TextStylizeState {
    fn default() -> Self {
        Self {
            style: UnicodeStyle::Bold,
            gradient: GradientMode::None,
            start_color: (255, 0, 0),
            end_color: (0, 0, 255),
            input_text: String::new(),
            cursor_position: 0,
            selected_setting: 0,
            editing_text: false,
        }
    }
}

impl TextStylizeState {
    pub fn settings_count() -> usize {
        6 // style, gradient, start_color, end_color, input, output_format
    }

    pub fn setting_name(&self, index: usize) -> &'static str {
        match index {
            0 => "Style",
            1 => "Gradient",
            2 => "Start Color",
            3 => "End Color",
            4 => "Input Text",
            _ => "Unknown",
        }
    }

    pub fn setting_value(&self, index: usize) -> String {
        match index {
            0 => self.style.name().to_string(),
            1 => self.gradient.name().to_string(),
            2 => format!(
                "#{:02X}{:02X}{:02X}",
                self.start_color.0, self.start_color.1, self.start_color.2
            ),
            3 => format!(
                "#{:02X}{:02X}{:02X}",
                self.end_color.0, self.end_color.1, self.end_color.2
            ),
            4 => {
                if self.input_text.is_empty() {
                    "[Type here...]".to_string()
                } else if self.input_text.len() > 20 {
                    format!("{}...", &self.input_text[..20])
                } else {
                    self.input_text.clone()
                }
            }
            _ => String::new(),
        }
    }
}

/// Main application state
pub struct AppState {
    // Mode and navigation
    pub current_mode: RenderMode,
    pub focus: FocusedWidget,
    pub show_help: bool,
    pub should_quit: bool,

    // Mode-specific state
    pub ascii_state: AsciiRenderState,
    pub unicode_state: UnicodeRenderState,
    pub text_state: TextStylizeState,

    // Shared state
    pub input_file: Option<PathBuf>,
    pub input_image: Option<Arc<DynamicImage>>,
    pub preview_content: Option<String>,
    pub preview_scroll: usize,
    pub preview_scroll_x: usize,  // Horizontal scroll position
    pub status_message: String,
    pub status_is_error: bool,

    // Terminal info
    pub terminal_size: (u16, u16),
    pub capabilities: TerminalCapabilities,

    // Performance
    pub perf_metrics: PerfMetrics,
    pub is_rendering: bool,

    // Configuration
    pub config: Config,

    // Interactive load prompt (when pressing [L])
    pub load_prompt_active: bool,
    pub load_prompt_input: String,
    pub load_prompt_error: Option<String>,

    // Preview control
    pub preview_output_format: OutputFormat,

    // Worker communication
    worker_tx: Sender<WorkerMessage>,
}

/// Output format used when saving previews
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Ansi,
    Html,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Ansi
    }
}

impl OutputFormat {
    pub fn name(&self) -> &'static str {
        match self {
            OutputFormat::Ansi => "ANSI",
            OutputFormat::Html => "HTML",
        }
    }
}

impl AppState {
    pub fn new(
        config: Config,
        capabilities: TerminalCapabilities,
        worker_tx: Sender<WorkerMessage>,
    ) -> Self {
        let (width, height) = capabilities.size;

        // Initialize from config
        let ascii_state = AsciiRenderState {
            charset: config.ascii.default_charset.clone(),
            width: config.ascii.default_width,
            invert: false,
            edge_enhance: config.ascii.edge_enhance,
            selected_setting: 0,
        };

        let unicode_state = UnicodeRenderState {
            mode: config.unicode.default_mode,
            width: config.unicode.default_width,
            color_mode: capabilities.color_support,
            selected_setting: 0,
        };

        Self {
            current_mode: RenderMode::default(),
            focus: FocusedWidget::default(),
            show_help: false,
            should_quit: false,

            ascii_state,
            unicode_state,
            text_state: TextStylizeState::default(),

            input_file: None,
            input_image: None,
            preview_content: None,
            preview_scroll: 0,
            preview_scroll_x: 0,
            status_message: "Ready - Press [?] for help".to_string(),
            status_is_error: false,

            terminal_size: (width, height),
            capabilities,

            perf_metrics: PerfMetrics::new(),
            is_rendering: false,

            config,

            // Load prompt defaults
            load_prompt_active: false,
            load_prompt_input: String::new(),
            load_prompt_error: None,

            // Default to ANSI output for terminal-first usage
            preview_output_format: OutputFormat::default(),

            worker_tx,
        }
    }

    /// Set current render mode
    pub fn set_mode(&mut self, mode: RenderMode) {
        if self.current_mode != mode {
            self.current_mode = mode;
            self.preview_content = None;
            self.preview_scroll = 0;
            self.set_status(&format!("Switched to {} mode", mode.name()), false);
        }
    }

    /// Update terminal size on resize
    pub fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    /// Set status message
    pub fn set_status(&mut self, message: &str, is_error: bool) {
        self.status_message = message.to_string();
        self.status_is_error = is_error;
    }

    /// Set the input image
    pub fn set_input_image(&mut self, path: PathBuf, image: DynamicImage) {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        self.input_file = Some(path);
        self.input_image = Some(Arc::new(image));
        self.set_status(&format!("Loaded: {}", filename), false);
        self.preview_content = None;

        // Clear any load prompt state
        self.load_prompt_active = false;
        self.load_prompt_input.clear();
        self.load_prompt_error = None;

        // Auto-render after loading
        self.trigger_render();
    }

    /// Trigger a render operation based on current mode
    pub fn trigger_render(&mut self) {
        if self.is_rendering {
            return;
        }

        match self.current_mode {
            RenderMode::ImageToAscii | RenderMode::ImageToUnicode => {
                // Clone the Arc first to avoid borrow conflicts
                let image = if let Some(img) = self.input_image.as_ref() {
                    Arc::clone(img)
                } else {
                    self.set_status("No image loaded - Press [L] to load", false);
                    return;
                };

                self.is_rendering = true;
                self.set_status("Rendering...", false);

                let msg = match self.current_mode {
                    RenderMode::ImageToAscii => WorkerMessage::AsciiRequest {
                        image,
                        width: self.ascii_state.width,
                        charset: self.ascii_state.charset.clone(),
                        invert: self.ascii_state.invert,
                        edge_enhance: self.ascii_state.edge_enhance,
                    },
                    RenderMode::ImageToUnicode => WorkerMessage::UnicodeRequest {
                        image,
                        width: self.unicode_state.width,
                        mode: self.unicode_state.mode,
                        color_mode: self.unicode_state.color_mode,
                    },
                    _ => return,
                };

                let _ = self.worker_tx.send(msg);
            }
            RenderMode::TextStylizer => {
                if self.text_state.input_text.is_empty() {
                    self.set_status("Enter text to stylize", false);
                    return;
                }

                self.is_rendering = true;
                self.set_status("Stylizing...", false);

                let msg = WorkerMessage::TextRequest {
                    text: self.text_state.input_text.clone(),
                    style: self.text_state.style,
                    gradient: self.text_state.gradient,
                    start_color: self.text_state.start_color,
                    end_color: self.text_state.end_color,
                };

                let _ = self.worker_tx.send(msg);
            }
        }
    }

    /// Handle response from worker thread
    pub fn handle_worker_response(&mut self, response: WorkerResponse) {
        self.is_rendering = false;

        match response {
            WorkerResponse::AsciiComplete { output, render_time } => {
                self.preview_content = Some(output);
                self.reset_scroll();
                self.perf_metrics.last_render_time_ms = render_time;
                self.set_status(&format!("Rendered in {}ms", render_time), false);
            }
            WorkerResponse::UnicodeComplete { output, render_time } => {
                self.preview_content = Some(output);
                self.reset_scroll();
                self.perf_metrics.last_render_time_ms = render_time;
                self.set_status(&format!("Rendered in {}ms", render_time), false);
            }
            WorkerResponse::TextComplete { output, render_time } => {
                self.preview_content = Some(output);
                self.reset_scroll();
                self.perf_metrics.last_render_time_ms = render_time;
                self.set_status(&format!("Stylized in {}ms", render_time), false);
            }
            WorkerResponse::Error(err) => {
                self.set_status(&format!("Error: {}", err), true);
            }
        }

        // If a load prompt was active and a worker returned an error, show the error message
        if let Some(err) = self.load_prompt_error.clone() {
            self.set_status(&err, true);
        }
    }

    /// Get current mode's selected setting index
    pub fn current_selected_setting(&self) -> usize {
        match self.current_mode {
            RenderMode::ImageToAscii => self.ascii_state.selected_setting,
            RenderMode::ImageToUnicode => self.unicode_state.selected_setting,
            RenderMode::TextStylizer => self.text_state.selected_setting,
        }
    }

    /// Start the interactive load prompt (shows a modal for path input)
    pub fn start_load_prompt(&mut self) {
        self.load_prompt_active = true;
        self.load_prompt_input.clear();
        self.load_prompt_error = None;
        self.set_status("Enter image path and press Enter", false);
    }

    /// Cancel the interactive load prompt
    pub fn cancel_load_prompt(&mut self) {
        self.load_prompt_active = false;
        self.load_prompt_input.clear();
        self.load_prompt_error = None;
        self.set_status("Load cancelled", false);
    }

    /// Attempt to load the image from the prompt input
    pub fn submit_load_prompt(&mut self) {
        let input = self.load_prompt_input.trim();
        if input.is_empty() {
            self.load_prompt_error = Some("Path is empty".to_string());
            self.set_status("Path is empty", true);
            return;
        }

        let path = PathBuf::from(input);
        if !path.exists() {
            self.load_prompt_error = Some("File not found".to_string());
            self.set_status("File not found", true);
            return;
        }

        match crate::image_loader::load_image(&path) {
            Ok(img) => {
                self.set_input_image(path, img);
            }
            Err(e) => {
                self.load_prompt_error = Some(format!("Failed to load: {}", e));
                self.set_status(&format!("Failed to load: {}", e), true);
            }
        }
    }

    /// Get current mode's settings count
    pub fn current_settings_count(&self) -> usize {
        match self.current_mode {
            RenderMode::ImageToAscii => AsciiRenderState::settings_count(),
            RenderMode::ImageToUnicode => UnicodeRenderState::settings_count(),
            RenderMode::TextStylizer => TextStylizeState::settings_count(),
        }
    }

    /// Navigate to next setting in control panel
    pub fn next_setting(&mut self) {
        let count = self.current_settings_count();
        match self.current_mode {
            RenderMode::ImageToAscii => {
                self.ascii_state.selected_setting =
                    (self.ascii_state.selected_setting + 1) % count;
            }
            RenderMode::ImageToUnicode => {
                self.unicode_state.selected_setting =
                    (self.unicode_state.selected_setting + 1) % count;
            }
            RenderMode::TextStylizer => {
                self.text_state.selected_setting = (self.text_state.selected_setting + 1) % count;
            }
        }
    }

    /// Navigate to previous setting in control panel
    pub fn prev_setting(&mut self) {
        let count = self.current_settings_count();
        match self.current_mode {
            RenderMode::ImageToAscii => {
                self.ascii_state.selected_setting = if self.ascii_state.selected_setting == 0 {
                    count - 1
                } else {
                    self.ascii_state.selected_setting - 1
                };
            }
            RenderMode::ImageToUnicode => {
                self.unicode_state.selected_setting = if self.unicode_state.selected_setting == 0 {
                    count - 1
                } else {
                    self.unicode_state.selected_setting - 1
                };
            }
            RenderMode::TextStylizer => {
                self.text_state.selected_setting = if self.text_state.selected_setting == 0 {
                    count - 1
                } else {
                    self.text_state.selected_setting - 1
                };
            }
        }
    }

    /// Scroll preview up
    pub fn scroll_up(&mut self, amount: usize) {
        self.preview_scroll = self.preview_scroll.saturating_sub(amount);
    }

    /// Scroll preview down
    pub fn scroll_down(&mut self, amount: usize) {
        if let Some(ref content) = self.preview_content {
            let line_count = content.lines().count();
            self.preview_scroll = (self.preview_scroll + amount).min(line_count.saturating_sub(1));
        }
    }

    /// Scroll preview left (horizontal)
    pub fn scroll_left(&mut self, amount: usize) {
        self.preview_scroll_x = self.preview_scroll_x.saturating_sub(amount);
    }

    /// Scroll preview right (horizontal)
    pub fn scroll_right(&mut self, amount: usize) {
        if let Some(ref content) = self.preview_content {
            // Find the maximum line width
            let max_width = content
                .lines()
                .map(|line| crate::input::strip_ansi_codes(line).chars().count())
                .max()
                .unwrap_or(0);
            self.preview_scroll_x = (self.preview_scroll_x + amount).min(max_width.saturating_sub(1));
        }
    }

    /// Reset horizontal scroll when content changes
    pub fn reset_scroll(&mut self) {
        self.preview_scroll = 0;
        self.preview_scroll_x = 0;
    }

    /// Adjust zoom (increase width when zooming out = more columns, or decrease width when zooming in = fewer columns)
    /// When `zoom_in` is true we *magnify* (reduce columns) else we *zoom out* (increase columns)
    pub fn adjust_zoom(&mut self, zoom_in: bool) {
        match self.current_mode {
            RenderMode::ImageToAscii => {
                let width = self.ascii_state.width;
                let new = zoom_step(width, zoom_in);
                self.ascii_state.width = new;
                self.set_status(&format!("ASCII width: {}", new), false);
                self.trigger_render();
            }
            RenderMode::ImageToUnicode => {
                let width = self.unicode_state.width;
                let new = zoom_step(width, zoom_in);
                self.unicode_state.width = new;
                self.set_status(&format!("Unicode width: {}", new), false);
                self.trigger_render();
            }
            RenderMode::TextStylizer => {
                // Text stylizer width not applicable
                self.set_status("Zoom not applicable for Text Stylizer", true);
            }
        }
    }
}

/// Compute a zoom step size for given width. Zoom in reduces width, zoom out increases width.
fn zoom_step(width: usize, zoom_in: bool) -> usize {
    if zoom_in {
        // Reduce width by ~20%, clamp to at least 10
        let new = (width * 4) / 5;
        new.max(10)
    } else {
        // Increase width by ~25%, clamp to a reasonable maximum
        let new = (width * 5) / 4;
        new.min(4000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_step_behaviour() {
        assert_eq!(zoom_step(80, true), 64); // zoom in
        assert_eq!(zoom_step(80, false), 100); // zoom out
        assert_eq!(zoom_step(12, true), 10); // min clamp
        assert!(zoom_step(4000, false) <= 4000);
    }
}
