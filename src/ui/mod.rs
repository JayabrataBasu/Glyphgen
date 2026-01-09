//! UI module
//!
//! Contains all UI rendering components using Ratatui.

mod help;
mod preview;
mod widgets;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::{AppState, FocusedWidget, RenderMode};

/// Main render function - draws the entire UI
pub fn render(frame: &mut Frame, state: &AppState) {
    let size = frame.area();

    // Check minimum size
    if size.width < 40 || size.height < 15 {
        render_size_warning(frame, size);
        return;
    }

    // Main layout: title bar, content, status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Min(10),    // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    render_title_bar(frame, main_chunks[0], state);
    render_main_content(frame, main_chunks[1], state);
    render_status_bar(frame, main_chunks[2], state);

    // Render help overlay if active
    if state.show_help {
        help::render_help_overlay(frame, size);
    }
}

/// Render warning when terminal is too small
fn render_size_warning(frame: &mut Frame, area: Rect) {
    let warning = Paragraph::new("Terminal too small!\nMinimum: 40x15")
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(warning, area);
}

/// Render the title bar
fn render_title_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let title = Line::from(vec![
        Span::styled(
            " Glyphgen ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v1.0", Style::default().fg(Color::DarkGray)),
        Span::raw(" │ "),
        Span::styled(
            state.current_mode.name(),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" ".repeat(
            (area.width as usize)
                .saturating_sub(35)
                .saturating_sub(state.current_mode.name().len()),
        )),
        Span::styled("[?]", Style::default().fg(Color::Green)),
        Span::raw(" Help  "),
        Span::styled("[Q]", Style::default().fg(Color::Red)),
        Span::raw(" Quit "),
    ]);

    let title_widget = Paragraph::new(title)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(title_widget, area);
}

/// Render the main content area
fn render_main_content(frame: &mut Frame, area: Rect, state: &AppState) {
    // Responsive layout: side-by-side if wide enough, stacked if narrow
    if area.width >= 80 {
        render_wide_layout(frame, area, state);
    } else {
        render_narrow_layout(frame, area, state);
    }
}

/// Render side-by-side layout for wide terminals
fn render_wide_layout(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25), // Left panel
            Constraint::Min(40),    // Preview area
        ])
        .split(area);

    // Left panel: mode selector + control panel
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Mode selector
            Constraint::Min(5),    // Control panel
        ])
        .split(chunks[0]);

    render_mode_selector(frame, left_chunks[0], state);
    render_control_panel(frame, left_chunks[1], state);
    preview::render_preview(frame, chunks[1], state);
}

/// Render stacked layout for narrow terminals
fn render_narrow_layout(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Mode selector
            Constraint::Length(8),  // Control panel
            Constraint::Min(5),     // Preview area
        ])
        .split(area);

    render_mode_selector(frame, chunks[0], state);
    render_control_panel(frame, chunks[1], state);
    preview::render_preview(frame, chunks[2], state);
}

/// Render the mode selector widget
fn render_mode_selector(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focus == FocusedWidget::ModeSelector;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            " Mode ",
            Style::default().add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let modes = RenderMode::all();
    let mut lines = Vec::new();

    for (idx, mode) in modes.iter().enumerate() {
        let is_selected = *mode == state.current_mode;
        let bullet = if is_selected { "●" } else { "○" };
        let shortcut = format!("[{}]", idx + 1);

        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", bullet),
                if is_selected {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::styled(mode.name(), style),
            Span::styled(format!(" {}", shortcut), Style::default().fg(Color::DarkGray)),
        ]));
    }

    let widget = Paragraph::new(lines);
    frame.render_widget(widget, inner);
}

/// Render the control panel
fn render_control_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focus == FocusedWidget::ControlPanel;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            " Settings ",
            Style::default().add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match state.current_mode {
        RenderMode::ImageToAscii => widgets::render_ascii_controls(frame, inner, state, is_focused),
        RenderMode::ImageToUnicode => {
            widgets::render_unicode_controls(frame, inner, state, is_focused)
        }
        RenderMode::TextStylizer => widgets::render_text_controls(frame, inner, state, is_focused),
    }
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let status_color = if state.status_is_error {
        Color::Red
    } else {
        Color::White
    };

    // Format performance metrics
    let perf_info = format!(
        "FPS: {:>3} │ Render: {:>4}ms",
        state.perf_metrics.fps_int(),
        state.perf_metrics.last_render_time_ms
    );

    let file_info = state
        .input_file
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| format!(" │ {}", n.to_string_lossy()))
        .unwrap_or_default();

    // Calculate spacing
    let status_len = state.status_message.len();
    let info_len = perf_info.len() + file_info.len();
    let spacing = (area.width as usize)
        .saturating_sub(status_len)
        .saturating_sub(info_len)
        .saturating_sub(2);

    let status = Line::from(vec![
        Span::raw(" "),
        Span::styled(&state.status_message, Style::default().fg(status_color)),
        Span::raw(" ".repeat(spacing)),
        Span::styled(&perf_info, Style::default().fg(Color::DarkGray)),
        Span::styled(&file_info, Style::default().fg(Color::Blue)),
        Span::raw(" "),
    ]);

    let widget = Paragraph::new(status)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    frame.render_widget(widget, area);
}
