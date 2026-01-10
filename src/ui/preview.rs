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
        render_preview_content(frame, inner, content, state.preview_scroll, state.preview_scroll_x);
    } else {
        render_placeholder(frame, inner, state);
    }
}

/// Convert a line with ANSI codes to a Ratatui Line with colored Spans, applying horizontal offset
fn ansi_line_to_ratatui_line(line: &str, h_offset: usize, max_width: usize) -> Line<'static> {
    let parts = crate::input::parse_ansi_to_spans(line);
    
    if parts.is_empty() {
        return Line::raw(String::new());
    }

    // Build spans with colors, then apply horizontal offset
    let mut all_chars: Vec<(char, Style)> = Vec::new();
    
    for (text, fg, bg) in parts {
        let mut style = Style::default();
        if let Some(fg_color) = fg {
            style = style.fg(fg_color);
        }
        if let Some(bg_color) = bg {
            style = style.bg(bg_color);
        }
        for c in text.chars() {
            all_chars.push((c, style));
        }
    }
    
    // Apply horizontal scroll offset
    let chars_to_show: Vec<(char, Style)> = all_chars
        .into_iter()
        .skip(h_offset)
        .take(max_width)
        .collect();
    
    if chars_to_show.is_empty() {
        return Line::raw(String::new());
    }
    
    // Group consecutive chars with same style into spans
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style = chars_to_show[0].1;
    
    for (c, style) in chars_to_show {
        if style == current_style {
            current_text.push(c);
        } else {
            if !current_text.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
            }
            current_text.push(c);
            current_style = style;
        }
    }
    
    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, current_style));
    }

    Line::from(spans)
}

/// Render the preview content with scrolling (vertical and horizontal) and ANSI color support
fn render_preview_content(frame: &mut Frame, area: Rect, content: &str, scroll_y: usize, scroll_x: usize) {
    let viewport_width = area.width as usize;
    
    let lines: Vec<Line> = content
        .lines()
        .skip(scroll_y)
        .take(area.height as usize)
        .map(|line| ansi_line_to_ratatui_line(line, scroll_x, viewport_width))
        .collect();

    let total_lines = content.lines().count();
    let visible_lines = area.height as usize;

    let widget = Paragraph::new(lines);

    frame.render_widget(widget, area);

    // Render vertical scrollbar if content is scrollable
    if total_lines > visible_lines {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_lines)
            .position(scroll_y)
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
