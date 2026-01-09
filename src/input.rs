//! Input handling
//!
//! Maps keyboard events to state transitions with context-sensitive bindings.

use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use crate::image_loader::load_image;
use crate::state::{AppState, FocusedWidget, RenderMode};

/// Handle an input event
pub fn handle_event(event: Event, state: &mut AppState) -> Result<()> {
    match event {
        Event::Key(key_event) => handle_key_event(key_event, state),
        Event::Resize(_, _) => Ok(()), // Already handled in main loop
        Event::Mouse(_) => Ok(()),      // Mouse support can be added later
        _ => Ok(()),
    }
}

/// Handle a key event
fn handle_key_event(key: KeyEvent, state: &mut AppState) -> Result<()> {
    // Handle help overlay
    if state.show_help {
        return handle_help_input(key, state);
    }

    // Handle interactive load prompt
    if state.load_prompt_active {
        return handle_load_prompt_input(key, state);
    }

    // Handle text input mode
    if state.text_state.editing_text {
        return handle_text_input(key, state);
    }

    // Global shortcuts
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            state.should_quit = true;
            return Ok(());
        }
        KeyCode::Char('?') => {
            state.show_help = true;
            return Ok(());
        }
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                state.focus = state.focus.prev();
            } else {
                state.focus = state.focus.next();
            }
            return Ok(());
        }
        KeyCode::Esc => {
            // Could be used to cancel operations
            return Ok(());
        }
        _ => {}
    }

    // Context-sensitive handling
    match state.focus {
        FocusedWidget::ModeSelector => handle_mode_selector_input(key, state),
        FocusedWidget::ControlPanel => handle_control_panel_input(key, state),
        FocusedWidget::Preview => handle_preview_input(key, state),
    }
}

/// Handle input when help overlay is shown
fn handle_help_input(key: KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Enter => {
            state.show_help = false;
        }
        _ => {}
    }
    Ok(())
}

/// Handle text input for text stylizer
fn handle_text_input(key: KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => {
            state.text_state.editing_text = false;
            if key.code == KeyCode::Enter && !state.text_state.input_text.is_empty() {
                state.trigger_render();
            }
        }
        KeyCode::Backspace => {
            state.text_state.input_text.pop();
        }
        KeyCode::Char(c) => {
            state.text_state.input_text.push(c);
        }
        _ => {}
    }
    Ok(())
}

/// Handle input for the interactive load prompt
fn handle_load_prompt_input(key: KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            state.cancel_load_prompt();
        }
        KeyCode::Enter => {
            state.submit_load_prompt();
        }
        KeyCode::Backspace => {
            state.load_prompt_input.pop();
        }
        KeyCode::Char(c) => {
            state.load_prompt_input.push(c);
        }
        _ => {}
    }
    Ok(())
}

/// Handle input for mode selector widget
fn handle_mode_selector_input(key: KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        // Mode shortcuts
        KeyCode::Char('1') => state.set_mode(RenderMode::ImageToAscii),
        KeyCode::Char('2') => state.set_mode(RenderMode::ImageToUnicode),
        KeyCode::Char('3') => state.set_mode(RenderMode::TextStylizer),

        // Arrow navigation
        KeyCode::Up | KeyCode::Char('k') => {
            let modes = RenderMode::all();
            let current_idx = modes
                .iter()
                .position(|m| *m == state.current_mode)
                .unwrap_or(0);
            let new_idx = if current_idx == 0 {
                modes.len() - 1
            } else {
                current_idx - 1
            };
            state.set_mode(modes[new_idx]);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let modes = RenderMode::all();
            let current_idx = modes
                .iter()
                .position(|m| *m == state.current_mode)
                .unwrap_or(0);
            let new_idx = (current_idx + 1) % modes.len();
            state.set_mode(modes[new_idx]);
        }

        KeyCode::Enter => {
            // Mode already selected
        }

        // Common actions
        KeyCode::Char('l') | KeyCode::Char('L') => {
            state.start_load_prompt();
        }
        KeyCode::Char(' ') => state.trigger_render(),
        KeyCode::Char('s') | KeyCode::Char('S') => save_output(state)?,

        _ => {}
    }
    Ok(())
}

/// Handle input for control panel widget
fn handle_control_panel_input(key: KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => state.prev_setting(),
        KeyCode::Down | KeyCode::Char('j') => state.next_setting(),

        // Adjust settings
        KeyCode::Left | KeyCode::Char('h') => adjust_setting_left(state),
        KeyCode::Right | KeyCode::Char('l') => adjust_setting_right(state),
        KeyCode::Char('+') | KeyCode::Char('=') => adjust_setting_increase(state),
        KeyCode::Char('-') | KeyCode::Char('_') => adjust_setting_decrease(state),

        // Toggle/action
        KeyCode::Char(' ') => {
            if !toggle_current_setting(state) {
                state.trigger_render();
            }
        }
        KeyCode::Enter => {
            // For text mode input
            if matches!(state.current_mode, RenderMode::TextStylizer)
                && state.text_state.selected_setting == 4
            {
                state.text_state.editing_text = true;
            } else {
                state.trigger_render();
            }
        }

        // Actions
        KeyCode::Char('L') => {
            state.start_load_prompt();
        }
        KeyCode::Char('S') => save_output(state)?,

        _ => {}
    }
    Ok(())
}

/// Handle input for preview widget
fn handle_preview_input(key: KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        // Scrolling
        KeyCode::Up | KeyCode::Char('k') => state.scroll_up(1),
        KeyCode::Down | KeyCode::Char('j') => state.scroll_down(1),
        KeyCode::PageUp => state.scroll_up(10),
        KeyCode::PageDown => state.scroll_down(10),
        KeyCode::Home => state.preview_scroll = 0,
        KeyCode::End => {
            if let Some(ref content) = state.preview_content {
                let line_count = content.lines().count();
                state.preview_scroll = line_count.saturating_sub(1);
            }
        }

        // Actions
        KeyCode::Char('s') | KeyCode::Char('S') => save_output(state)?,
        KeyCode::Char('c') | KeyCode::Char('C') => copy_to_clipboard(state)?,
        KeyCode::Char('l') | KeyCode::Char('L') => {
            state.start_load_prompt();
        }
        KeyCode::Char(' ') => state.trigger_render(),

        _ => {}
    }
    Ok(())
}

/// Adjust setting to the left (previous option)
fn adjust_setting_left(state: &mut AppState) {
    match state.current_mode {
        RenderMode::ImageToAscii => match state.ascii_state.selected_setting {
            1 => state.ascii_state.charset = state.ascii_state.charset.prev(),
            _ => {}
        },
        RenderMode::ImageToUnicode => match state.unicode_state.selected_setting {
            1 => state.unicode_state.mode = state.unicode_state.mode.prev(),
            2 => state.unicode_state.color_mode = state.unicode_state.color_mode.prev(),
            _ => {}
        },
        RenderMode::TextStylizer => match state.text_state.selected_setting {
            0 => state.text_state.style = state.text_state.style.prev(),
            1 => state.text_state.gradient = state.text_state.gradient.prev(),
            _ => {}
        },
    }
}

/// Adjust setting to the right (next option)
fn adjust_setting_right(state: &mut AppState) {
    match state.current_mode {
        RenderMode::ImageToAscii => match state.ascii_state.selected_setting {
            1 => state.ascii_state.charset = state.ascii_state.charset.next(),
            _ => {}
        },
        RenderMode::ImageToUnicode => match state.unicode_state.selected_setting {
            1 => state.unicode_state.mode = state.unicode_state.mode.next(),
            2 => state.unicode_state.color_mode = state.unicode_state.color_mode.next(),
            _ => {}
        },
        RenderMode::TextStylizer => match state.text_state.selected_setting {
            0 => state.text_state.style = state.text_state.style.next(),
            1 => state.text_state.gradient = state.text_state.gradient.next(),
            _ => {}
        },
    }
}

/// Increase numeric setting
fn adjust_setting_increase(state: &mut AppState) {
    match state.current_mode {
        RenderMode::ImageToAscii => {
            if state.ascii_state.selected_setting == 0 {
                state.ascii_state.width = (state.ascii_state.width + 10).min(300);
            }
        }
        RenderMode::ImageToUnicode => {
            if state.unicode_state.selected_setting == 0 {
                state.unicode_state.width = (state.unicode_state.width + 10).min(300);
            }
        }
        _ => {}
    }
}

/// Decrease numeric setting
fn adjust_setting_decrease(state: &mut AppState) {
    match state.current_mode {
        RenderMode::ImageToAscii => {
            if state.ascii_state.selected_setting == 0 {
                state.ascii_state.width = state.ascii_state.width.saturating_sub(10).max(20);
            }
        }
        RenderMode::ImageToUnicode => {
            if state.unicode_state.selected_setting == 0 {
                state.unicode_state.width = state.unicode_state.width.saturating_sub(10).max(20);
            }
        }
        _ => {}
    }
}

/// Toggle boolean setting, returns true if a setting was toggled
fn toggle_current_setting(state: &mut AppState) -> bool {
    match state.current_mode {
        RenderMode::ImageToAscii => match state.ascii_state.selected_setting {
            2 => {
                state.ascii_state.invert = !state.ascii_state.invert;
                true
            }
            3 => {
                state.ascii_state.edge_enhance = !state.ascii_state.edge_enhance;
                true
            }
            _ => false,
        },
        RenderMode::TextStylizer => {
            if state.text_state.selected_setting == 4 {
                state.text_state.editing_text = true;
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Open file dialog to load an image
fn load_image_dialog(state: &mut AppState) -> Result<()> {
    // Simple approach: use environment variable or hardcoded test path
    // In a real app, you'd integrate with a file picker or use stdin
    
    // Try to get path from environment or use a dialog
    if let Ok(path) = std::env::var("GLYPHGEN_IMAGE") {
        let path = PathBuf::from(path);
        match load_image(&path) {
            Ok(img) => {
                state.set_input_image(path, img);
            }
            Err(e) => {
                state.set_status(&format!("Failed to load: {}", e), true);
            }
        }
        return Ok(());
    }

    // Prompt user for path
    state.set_status("Enter image path in GLYPHGEN_IMAGE env var", false);
    
    // Alternative: Try common test images
    let test_paths = [
        "test.png",
        "test.jpg",
        "image.png",
        "image.jpg",
        "input.png",
        "input.jpg",
    ];
    
    for path_str in test_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            match load_image(&path) {
                Ok(img) => {
                    state.set_input_image(path, img);
                    return Ok(());
                }
                Err(_) => continue,
            }
        }
    }
    
    state.set_status("No image found. Set GLYPHGEN_IMAGE=path/to/image", false);
    Ok(())
}

/// Save output to file
fn save_output(state: &mut AppState) -> Result<()> {
    if let Some(ref content) = state.preview_content {
        let filename = match state.current_mode {
            RenderMode::ImageToAscii => "ascii_output.txt",
            RenderMode::ImageToUnicode => "unicode_output.txt",
            RenderMode::TextStylizer => "styled_text.txt",
        };

        // Strip ANSI codes for clean text output
        let clean_content = strip_ansi_codes(content);

        match std::fs::write(filename, &clean_content) {
            Ok(_) => {
                state.set_status(&format!("Saved to {}", filename), false);
            }
            Err(e) => {
                state.set_status(&format!("Save failed: {}", e), true);
            }
        }
    } else {
        state.set_status("Nothing to save - render first", false);
    }
    Ok(())
}

/// Copy output to clipboard
fn copy_to_clipboard(state: &mut AppState) -> Result<()> {
    if let Some(ref content) = state.preview_content {
        let clean_content = strip_ansi_codes(content);

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(&clean_content) {
                Ok(_) => {
                    state.set_status("Copied to clipboard", false);
                }
                Err(e) => {
                    state.set_status(&format!("Copy failed: {}", e), true);
                }
            },
            Err(e) => {
                state.set_status(&format!("Clipboard unavailable: {}", e), true);
            }
        }
    } else {
        state.set_status("Nothing to copy - render first", false);
    }
    Ok(())
}

/// Strip ANSI escape codes from text
fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_codes() {
        let input = "\x1b[38;2;255;0;0mRed\x1b[0m";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "Red");
    }

    #[test]
    fn test_strip_ansi_preserves_text() {
        let input = "Hello, World!";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "Hello, World!");
    }

    #[test]
    fn test_strip_ansi_complex() {
        let input = "\x1b[38;5;196mR\x1b[38;5;208mA\x1b[38;5;226mI\x1b[38;5;46mN\x1b[38;5;21mB\x1b[38;5;129mO\x1b[38;5;196mW\x1b[0m";
        let output = strip_ansi_codes(input);
        assert_eq!(output, "RAINBOW");
    }
}
