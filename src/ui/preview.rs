//! Preview area rendering

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use crate::state::{AppState, FocusedWidget};

/// Render the preview area
pub fn render_preview(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_focused = state.focus == FocusedWidget::Preview;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            " Preview ",
            Style::default().add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(ref content) = state.preview_content {
        render_preview_content(frame, inner, content, state.preview_scroll);
    } else {
        render_placeholder(frame, inner, state);
    }
}

/// Render the preview content with scrolling
fn render_preview_content(frame: &mut Frame, area: Rect, content: &str, scroll: usize) {
    let lines: Vec<Line> = content
        .lines()
        .skip(scroll)
        .take(area.height as usize)
        .map(|line| {
            // Strip ANSI escape sequences so raw control codes don't appear in the TUI preview
            let clean = crate::input::strip_ansi_codes(line);
            Line::raw(clean)
        })
        .collect();

    let total_lines = content.lines().count();
    let visible_lines = area.height as usize;

    let widget = Paragraph::new(lines).style(Style::default().fg(Color::White));

    frame.render_widget(widget, area);

    // Render scrollbar if content is scrollable
    if total_lines > visible_lines {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_lines)
            .position(scroll)
            .viewport_content_length(visible_lines);

        let scrollbar_area = Rect {
            x: area.x + area.width.saturating_sub(1),
            y: area.y,
            width: 1,
            height: area.height,
        };

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

/// Render placeholder when no content
fn render_placeholder(frame: &mut Frame, area: Rect, state: &AppState) {
    let message = match state.current_mode {
        crate::state::RenderMode::ImageToAscii | crate::state::RenderMode::ImageToUnicode => {
            if state.input_image.is_some() {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press [Space] to render",
                        Style::default().fg(Color::Yellow),
                    )),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "No image loaded",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press [L] to load an image",
                        Style::default().fg(Color::Green),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Supported formats: PNG, JPEG, GIF, WebP, BMP",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            }
        }
        crate::state::RenderMode::TextStylizer => {
            if state.text_state.input_text.is_empty() {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "No text to stylize",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Navigate to 'Input Text' and type your text",
                        Style::default().fg(Color::Green),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press [Enter] to stylize",
                        Style::default().fg(Color::Yellow),
                    )),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "Press [Space] to stylize text",
                        Style::default().fg(Color::Yellow),
                    )),
                ]
            }
        }
    };

    let widget = Paragraph::new(message)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(widget, area);
}
