//! Help overlay rendering

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the help overlay
pub fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Calculate overlay size (60% width, 70% height, centered)
    let overlay_width = (area.width as f32 * 0.7).min(70.0) as u16;
    let overlay_height = (area.height as f32 * 0.8).min(30.0) as u16;

    let overlay_area = centered_rect(overlay_width, overlay_height, area);

    // Clear background
    frame.render_widget(Clear, overlay_area);

    // Render help content
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Keyboard Shortcuts ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let help_text = create_help_text();
    let widget = Paragraph::new(help_text).style(Style::default().fg(Color::White));

    frame.render_widget(widget, inner);
}

/// Create help text content
fn create_help_text() -> Vec<Line<'static>> {
    let section_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default().fg(Color::Green);
    let desc_style = Style::default().fg(Color::White);

    vec![
        Line::from(Span::styled("Global", section_style)),
        Line::from(vec![
            Span::styled("  Q           ", key_style),
            Span::styled("Quit application", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ?           ", key_style),
            Span::styled("Toggle help overlay", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Tab         ", key_style),
            Span::styled("Next widget", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab   ", key_style),
            Span::styled("Previous widget", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Esc         ", key_style),
            Span::styled("Cancel / Close overlay", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("Mode Selector", section_style)),
        Line::from(vec![
            Span::styled("  1, 2, 3     ", key_style),
            Span::styled("Jump to mode", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ↑ ↓         ", key_style),
            Span::styled("Navigate modes", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Enter       ", key_style),
            Span::styled("Select mode", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("Control Panel", section_style)),
        Line::from(vec![
            Span::styled("  ↑ ↓         ", key_style),
            Span::styled("Navigate settings", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ← →         ", key_style),
            Span::styled("Adjust selection", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  + -         ", key_style),
            Span::styled("Adjust numeric values", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Space       ", key_style),
            Span::styled("Toggle / Render", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  L           ", key_style),
            Span::styled("Load image", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  S           ", key_style),
            Span::styled("Save output", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("Preview Area", section_style)),
        Line::from(vec![
            Span::styled("  ↑ ↓         ", key_style),
            Span::styled("Scroll by line", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  PgUp PgDn   ", key_style),
            Span::styled("Scroll by page", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Home End    ", key_style),
            Span::styled("Jump to top/bottom", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  C           ", key_style),
            Span::styled("Copy to clipboard", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "       [Press ? or Esc to close]",
            Style::default().fg(Color::DarkGray),
        )),
    ]
}

/// Create a centered rectangle
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal_padding = area.width.saturating_sub(width) / 2;
    let vertical_padding = area.height.saturating_sub(height) / 2;

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horizontal_padding),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1])[1]
}
