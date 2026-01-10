//! Control panel widgets for each mode

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::state::AppState;

/// Render ASCII mode control panel
pub fn render_ascii_controls(frame: &mut Frame, area: Rect, state: &AppState, is_focused: bool) {
    let selected = state.ascii_state.selected_setting;
    let mut lines = Vec::new();

    // Width setting
    lines.push(create_setting_line(
        "Width",
        &format!("{}", state.ascii_state.width),
        selected == 0 && is_focused,
        Some("[+/-]"),
    ));

    // Charset setting
    lines.push(create_setting_line(
        "Charset",
        state.ascii_state.charset.name(),
        selected == 1 && is_focused,
        Some("[←/→]"),
    ));

    // Invert setting
    lines.push(create_setting_line(
        "Invert",
        if state.ascii_state.invert { "On" } else { "Off" },
        selected == 2 && is_focused,
        Some("[Space]"),
    ));

    // Edge enhance setting
    lines.push(create_setting_line(
        "Edge Enhance",
        if state.ascii_state.edge_enhance {
            "On"
        } else {
            "Off"
        },
        selected == 3 && is_focused,
        Some("[Space]"),
    ));

    // Action buttons
    lines.push(Line::from(""));
    lines.push(create_action_line("[Space]", "Render"));
    lines.push(create_action_line("[L]", "Load Image"));
    lines.push(create_action_line("[S]", "Save Output"));

    // Tip shown when this panel is focused
    if is_focused {
        lines.push(Line::from(Span::styled(
            "Tip: Press Tab to switch focus between Mode Selector, Control Panel and Preview",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let widget = Paragraph::new(lines);
    frame.render_widget(widget, area);
}

/// Render Unicode mode control panel
pub fn render_unicode_controls(frame: &mut Frame, area: Rect, state: &AppState, is_focused: bool) {
    let selected = state.unicode_state.selected_setting;
    let mut lines = Vec::new();

    // Width setting
    lines.push(create_setting_line(
        "Width",
        &format!("{}", state.unicode_state.width),
        selected == 0 && is_focused,
        Some("[+/-]"),
    ));

    // Mode setting
    lines.push(create_setting_line(
        "Mode",
        state.unicode_state.mode.name(),
        selected == 1 && is_focused,
        Some("[←/→]"),
    ));

    // Color mode setting
    lines.push(create_setting_line(
        "Color",
        state.unicode_state.color_mode.name(),
        selected == 2 && is_focused,
        Some("[←/→]"),
    ));

    // Action buttons
    lines.push(Line::from(""));
    lines.push(create_action_line("[Space]", "Render"));
    lines.push(create_action_line("[L]", "Load Image"));
    lines.push(create_action_line("[S]", "Save Output"));

    // Tip shown when this panel is focused
    if is_focused {
        lines.push(Line::from(Span::styled(
            "Tip: Press Tab to switch focus between Mode Selector, Control Panel and Preview",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let widget = Paragraph::new(lines);
    frame.render_widget(widget, area);
}

/// Render Text Stylizer mode control panel
pub fn render_text_controls(frame: &mut Frame, area: Rect, state: &AppState, is_focused: bool) {
    let selected = state.text_state.selected_setting;
    let mut lines = Vec::new();

    // Style setting
    lines.push(create_setting_line(
        "Style",
        state.text_state.style.name(),
        selected == 0 && is_focused,
        Some("[←/→]"),
    ));

    // Gradient setting
    lines.push(create_setting_line(
        "Gradient",
        state.text_state.gradient.name(),
        selected == 1 && is_focused,
        Some("[←/→]"),
    ));

    // Start color
    let start_color = format!(
        "#{:02X}{:02X}{:02X}",
        state.text_state.start_color.0,
        state.text_state.start_color.1,
        state.text_state.start_color.2
    );
    lines.push(create_setting_line(
        "Start Color",
        &start_color,
        selected == 2 && is_focused,
        None,
    ));

    // End color
    let end_color = format!(
        "#{:02X}{:02X}{:02X}",
        state.text_state.end_color.0,
        state.text_state.end_color.1,
        state.text_state.end_color.2
    );
    lines.push(create_setting_line(
        "End Color",
        &end_color,
        selected == 3 && is_focused,
        None,
    ));

    // Input text
    let input_display = if state.text_state.editing_text {
        format!("{}▌", &state.text_state.input_text)
    } else if state.text_state.input_text.is_empty() {
        "[Type here...]".to_string()
    } else if state.text_state.input_text.len() > 15 {
        format!("{}...", &state.text_state.input_text[..15])
    } else {
        state.text_state.input_text.clone()
    };

    let is_input_selected = selected == 4 && is_focused;
    lines.push(Line::from(vec![
        Span::styled(
            if is_input_selected { "▸ " } else { "  " },
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            "Input: ",
            if is_input_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            },
        ),
        Span::styled(
            &input_display,
            if state.text_state.editing_text {
                Style::default().fg(Color::Green)
            } else if is_input_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ]));

    // Action buttons
    lines.push(Line::from(""));
    lines.push(create_action_line("[Space]", "Stylize"));
    lines.push(create_action_line("[S]", "Save Output"));

    // Tip shown when this panel is focused
    if is_focused {
        lines.push(Line::from(Span::styled(
            "Tip: Press Tab to switch focus between Mode Selector, Control Panel and Preview",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let widget = Paragraph::new(lines);
    frame.render_widget(widget, area);
}

/// Create a setting line with label, value, and optional hint
fn create_setting_line(
    label: &str,
    value: &str,
    is_selected: bool,
    hint: Option<&str>,
) -> Line<'static> {
    let indicator = if is_selected { "▸" } else { " " };
    let indicator_style = Style::default().fg(Color::Cyan);

    let label_style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let value_style = if is_selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mut spans = vec![
        Span::styled(format!("{} ", indicator), indicator_style),
        Span::styled(format!("{}: ", label), label_style),
        Span::styled(value.to_string(), value_style),
    ];

    if let Some(hint_text) = hint {
        spans.push(Span::styled(
            format!(" {}", hint_text),
            Style::default().fg(Color::DarkGray),
        ));
    }

    Line::from(spans)
}

/// Create an action line (button-like)
fn create_action_line(key: &str, label: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(key.to_string(), Style::default().fg(Color::Green)),
        Span::styled(format!(" {}", label), Style::default().fg(Color::White)),
    ])
}
