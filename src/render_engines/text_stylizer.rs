//! Text stylizer engine
//!
//! Converts plain text to Unicode stylized text with gradient coloring.

use anyhow::Result;
use unicode_segmentation::UnicodeSegmentation;

use crate::color_space::{interpolate_color, format_fg_color, Rgb, ANSI_RESET};
use crate::terminal_capabilities::ColorSupport;

/// Unicode text styles using Mathematical Alphanumeric Symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnicodeStyle {
    /// 𝐀𝐁𝐂 (U+1D400)
    #[default]
    Bold,
    /// 𝐴𝐵𝐶 (U+1D434)
    Italic,
    /// 𝑨𝑩𝑪 (U+1D468)
    BoldItalic,
    /// 𝒜ℬ𝒞 (U+1D49C)
    Script,
    /// 𝓐𝓑𝓒 (U+1D4D0)
    BoldScript,
    /// 𝔄𝔅ℭ (U+1D504)
    Fraktur,
    /// 𝔸𝔹ℂ (U+1D538)
    DoubleStruck,
    /// 𝖠𝖡𝖢 (U+1D5A0)
    SansSerif,
    /// 𝗔𝗕𝗖 (U+1D5D4)
    SansSerifBold,
    /// 𝙰𝙱𝙲 (U+1D670)
    Monospace,
    /// ＡＢＣ (U+FF21)
    Fullwidth,
    /// ⒶⒷⒸ (U+24B6)
    Circled,
    /// 🅐🅑🅒 (U+1F150)
    NegativeCircled,
    /// 🄰🄱🄲 (U+1F130)
    Squared,
}

impl UnicodeStyle {
    pub fn name(&self) -> &str {
        match self {
            UnicodeStyle::Bold => "Bold",
            UnicodeStyle::Italic => "Italic",
            UnicodeStyle::BoldItalic => "Bold Italic",
            UnicodeStyle::Script => "Script",
            UnicodeStyle::BoldScript => "Bold Script",
            UnicodeStyle::Fraktur => "Fraktur",
            UnicodeStyle::DoubleStruck => "Double-Struck",
            UnicodeStyle::SansSerif => "Sans-Serif",
            UnicodeStyle::SansSerifBold => "Sans-Serif Bold",
            UnicodeStyle::Monospace => "Monospace",
            UnicodeStyle::Fullwidth => "Fullwidth",
            UnicodeStyle::Circled => "Circled",
            UnicodeStyle::NegativeCircled => "Negative Circled",
            UnicodeStyle::Squared => "Squared",
        }
    }

    pub fn all() -> &'static [UnicodeStyle] {
        &[
            UnicodeStyle::Bold,
            UnicodeStyle::Italic,
            UnicodeStyle::BoldItalic,
            UnicodeStyle::Script,
            UnicodeStyle::BoldScript,
            UnicodeStyle::Fraktur,
            UnicodeStyle::DoubleStruck,
            UnicodeStyle::SansSerif,
            UnicodeStyle::SansSerifBold,
            UnicodeStyle::Monospace,
            UnicodeStyle::Fullwidth,
            UnicodeStyle::Circled,
            UnicodeStyle::NegativeCircled,
            UnicodeStyle::Squared,
        ]
    }

    pub fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|s| s == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    pub fn prev(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|s| s == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }

    /// Get the Unicode offset for uppercase letters
    fn uppercase_base(&self) -> Option<u32> {
        match self {
            UnicodeStyle::Bold => Some(0x1D400),
            UnicodeStyle::Italic => Some(0x1D434),
            UnicodeStyle::BoldItalic => Some(0x1D468),
            UnicodeStyle::Script => Some(0x1D49C),
            UnicodeStyle::BoldScript => Some(0x1D4D0),
            UnicodeStyle::Fraktur => Some(0x1D504),
            UnicodeStyle::DoubleStruck => Some(0x1D538),
            UnicodeStyle::SansSerif => Some(0x1D5A0),
            UnicodeStyle::SansSerifBold => Some(0x1D5D4),
            UnicodeStyle::Monospace => Some(0x1D670),
            UnicodeStyle::Fullwidth => Some(0xFF21),
            UnicodeStyle::Circled => Some(0x24B6),
            UnicodeStyle::NegativeCircled => Some(0x1F150),
            UnicodeStyle::Squared => Some(0x1F130),
        }
    }

    /// Get the Unicode offset for lowercase letters
    fn lowercase_base(&self) -> Option<u32> {
        match self {
            UnicodeStyle::Bold => Some(0x1D41A),
            UnicodeStyle::Italic => Some(0x1D44E),
            UnicodeStyle::BoldItalic => Some(0x1D482),
            UnicodeStyle::Script => Some(0x1D4B6),
            UnicodeStyle::BoldScript => Some(0x1D4EA),
            UnicodeStyle::Fraktur => Some(0x1D51E),
            UnicodeStyle::DoubleStruck => Some(0x1D552),
            UnicodeStyle::SansSerif => Some(0x1D5BA),
            UnicodeStyle::SansSerifBold => Some(0x1D5EE),
            UnicodeStyle::Monospace => Some(0x1D68A),
            UnicodeStyle::Fullwidth => Some(0xFF41),
            UnicodeStyle::Circled => Some(0x24D0),
            UnicodeStyle::NegativeCircled => None, // No lowercase
            UnicodeStyle::Squared => None,         // No lowercase
        }
    }

    /// Get the Unicode offset for digits
    fn digit_base(&self) -> Option<u32> {
        match self {
            UnicodeStyle::Bold => Some(0x1D7CE),
            UnicodeStyle::DoubleStruck => Some(0x1D7D8),
            UnicodeStyle::SansSerif => Some(0x1D7E2),
            UnicodeStyle::SansSerifBold => Some(0x1D7EC),
            UnicodeStyle::Monospace => Some(0x1D7F6),
            UnicodeStyle::Fullwidth => Some(0xFF10),
            UnicodeStyle::Circled => Some(0x2460), // ①②③ (1-9 only)
            _ => None,
        }
    }
}

/// Gradient coloring mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradientMode {
    #[default]
    None,
    Horizontal,
    Rainbow,
}

impl GradientMode {
    pub fn name(&self) -> &str {
        match self {
            GradientMode::None => "None",
            GradientMode::Horizontal => "Horizontal",
            GradientMode::Rainbow => "Rainbow",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            GradientMode::None => GradientMode::Horizontal,
            GradientMode::Horizontal => GradientMode::Rainbow,
            GradientMode::Rainbow => GradientMode::None,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            GradientMode::None => GradientMode::Rainbow,
            GradientMode::Horizontal => GradientMode::None,
            GradientMode::Rainbow => GradientMode::Horizontal,
        }
    }
}

/// Stylize text with Unicode styles and optional gradient
pub fn stylize_text(
    text: &str,
    style: UnicodeStyle,
    gradient: GradientMode,
    start_color: (u8, u8, u8),
    end_color: (u8, u8, u8),
) -> Result<String> {
    // First apply Unicode style
    let styled = apply_unicode_style(text, style);

    // Then apply gradient if requested
    let colored = apply_gradient(&styled, gradient, start_color, end_color);

    Ok(colored)
}

/// Apply Unicode style transformation to text
fn apply_unicode_style(text: &str, style: UnicodeStyle) -> String {
    let mut result = String::with_capacity(text.len() * 4); // Unicode chars can be 4 bytes

    for c in text.chars() {
        result.push(transform_char(c, style));
    }

    result
}

/// Transform a single character to the styled version
fn transform_char(c: char, style: UnicodeStyle) -> char {
    // Handle uppercase letters
    if c.is_ascii_uppercase() {
        if let Some(base) = style.uppercase_base() {
            let offset = c as u32 - 'A' as u32;
            // Handle exceptions in some Unicode ranges
            let code = match style {
                UnicodeStyle::Script if c == 'B' => 0x212C, // ℬ
                UnicodeStyle::Script if c == 'E' => 0x2130, // ℰ
                UnicodeStyle::Script if c == 'F' => 0x2131, // ℱ
                UnicodeStyle::Script if c == 'H' => 0x210B, // ℋ
                UnicodeStyle::Script if c == 'I' => 0x2110, // ℐ
                UnicodeStyle::Script if c == 'L' => 0x2112, // ℒ
                UnicodeStyle::Script if c == 'M' => 0x2133, // ℳ
                UnicodeStyle::Script if c == 'R' => 0x211B, // ℛ
                UnicodeStyle::Fraktur if c == 'C' => 0x212D, // ℭ
                UnicodeStyle::Fraktur if c == 'H' => 0x210C, // ℌ
                UnicodeStyle::Fraktur if c == 'I' => 0x2111, // ℑ
                UnicodeStyle::Fraktur if c == 'R' => 0x211C, // ℜ
                UnicodeStyle::Fraktur if c == 'Z' => 0x2128, // ℨ
                UnicodeStyle::DoubleStruck if c == 'C' => 0x2102, // ℂ
                UnicodeStyle::DoubleStruck if c == 'H' => 0x210D, // ℍ
                UnicodeStyle::DoubleStruck if c == 'N' => 0x2115, // ℕ
                UnicodeStyle::DoubleStruck if c == 'P' => 0x2119, // ℙ
                UnicodeStyle::DoubleStruck if c == 'Q' => 0x211A, // ℚ
                UnicodeStyle::DoubleStruck if c == 'R' => 0x211D, // ℝ
                UnicodeStyle::DoubleStruck if c == 'Z' => 0x2124, // ℤ
                _ => base + offset,
            };
            return char::from_u32(code).unwrap_or(c);
        }
    }

    // Handle lowercase letters
    if c.is_ascii_lowercase() {
        if let Some(base) = style.lowercase_base() {
            let offset = c as u32 - 'a' as u32;
            // Handle exceptions
            let code = match style {
                UnicodeStyle::Script if c == 'e' => 0x212F, // ℯ
                UnicodeStyle::Script if c == 'g' => 0x210A, // ℊ
                UnicodeStyle::Script if c == 'o' => 0x2134, // ℴ
                UnicodeStyle::Italic if c == 'h' => 0x210E, // ℎ (Planck constant)
                _ => base + offset,
            };
            return char::from_u32(code).unwrap_or(c);
        } else {
            // For styles without lowercase, use uppercase
            return transform_char(c.to_ascii_uppercase(), style);
        }
    }

    // Handle digits
    if c.is_ascii_digit() {
        if let Some(base) = style.digit_base() {
            let offset = c as u32 - '0' as u32;
            // Circled digits are special (① starts at 1, ⓪ is 0x24EA)
            let code = if matches!(style, UnicodeStyle::Circled) {
                if c == '0' {
                    0x24EA // ⓪
                } else {
                    base + offset - 1 // ①②③... start at offset-1
                }
            } else {
                base + offset
            };
            return char::from_u32(code).unwrap_or(c);
        }
    }

    // Return unchanged for non-transformable characters
    c
}

/// Apply gradient coloring to text
fn apply_gradient(
    text: &str,
    mode: GradientMode,
    start_color: (u8, u8, u8),
    end_color: (u8, u8, u8),
) -> String {
    match mode {
        GradientMode::None => text.to_string(),
        GradientMode::Horizontal => apply_horizontal_gradient(text, start_color, end_color),
        GradientMode::Rainbow => apply_rainbow_gradient(text),
    }
}

/// Apply horizontal gradient (interpolate from start to end color)
fn apply_horizontal_gradient(
    text: &str,
    start: (u8, u8, u8),
    end: (u8, u8, u8),
) -> String {
    let graphemes: Vec<&str> = text.graphemes(true).collect();
    let len = graphemes.len();

    if len == 0 {
        return String::new();
    }

    let start_rgb = Rgb::from_tuple(start);
    let end_rgb = Rgb::from_tuple(end);

    let mut result = String::with_capacity(text.len() * 20);

    for (i, grapheme) in graphemes.iter().enumerate() {
        let t = if len > 1 {
            i as f32 / (len - 1) as f32
        } else {
            0.0
        };

        let color = interpolate_color(start_rgb, end_rgb, t);

        // Skip coloring for whitespace
        if grapheme.chars().all(|c| c.is_whitespace()) {
            result.push_str(grapheme);
        } else {
            result.push_str(&format_fg_color(color, ColorSupport::TrueColor));
            result.push_str(grapheme);
            result.push_str(ANSI_RESET);
        }
    }

    result
}

/// Apply rainbow gradient
fn apply_rainbow_gradient(text: &str) -> String {
    let graphemes: Vec<&str> = text.graphemes(true).collect();
    let len = graphemes.len();

    if len == 0 {
        return String::new();
    }

    let mut result = String::with_capacity(text.len() * 20);

    for (i, grapheme) in graphemes.iter().enumerate() {
        // Calculate hue (0-360) based on position
        let hue = (i as f32 / len as f32) * 360.0;
        let rgb = hue_to_rgb(hue);

        // Skip coloring for whitespace
        if grapheme.chars().all(|c| c.is_whitespace()) {
            result.push_str(grapheme);
        } else {
            result.push_str(&format_fg_color(rgb, ColorSupport::TrueColor));
            result.push_str(grapheme);
            result.push_str(ANSI_RESET);
        }
    }

    result
}

/// Convert hue (0-360) to RGB
fn hue_to_rgb(hue: f32) -> Rgb {
    let h = hue / 60.0;
    let c = 1.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());

    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Rgb::new(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_cycling() {
        let style = UnicodeStyle::Bold;
        assert_eq!(style.next(), UnicodeStyle::Italic);
        assert_eq!(style.prev(), UnicodeStyle::Squared);
    }

    #[test]
    fn test_gradient_cycling() {
        let mode = GradientMode::None;
        assert_eq!(mode.next(), GradientMode::Horizontal);
        assert_eq!(mode.prev(), GradientMode::Rainbow);
    }

    #[test]
    fn test_bold_transformation() {
        let styled = apply_unicode_style("ABC", UnicodeStyle::Bold);
        assert_eq!(styled, "𝐀𝐁𝐂");
    }

    #[test]
    fn test_bold_lowercase() {
        let styled = apply_unicode_style("abc", UnicodeStyle::Bold);
        assert_eq!(styled, "𝐚𝐛𝐜");
    }

    #[test]
    fn test_double_struck() {
        let styled = apply_unicode_style("CNQRZ", UnicodeStyle::DoubleStruck);
        // These should use the special characters
        assert!(styled.contains('ℂ'));
        assert!(styled.contains('ℕ'));
    }

    #[test]
    fn test_circled() {
        let styled = apply_unicode_style("ABC", UnicodeStyle::Circled);
        assert_eq!(styled, "ⒶⒷⒸ");
    }

    #[test]
    fn test_preserves_spaces() {
        let styled = apply_unicode_style("A B C", UnicodeStyle::Bold);
        assert_eq!(styled, "𝐀 𝐁 𝐂");
    }

    #[test]
    fn test_preserves_punctuation() {
        let styled = apply_unicode_style("Hello, World!", UnicodeStyle::Bold);
        assert!(styled.contains(','));
        assert!(styled.contains('!'));
    }

    #[test]
    fn test_stylize_with_gradient() {
        let result = stylize_text(
            "Test",
            UnicodeStyle::Bold,
            GradientMode::Horizontal,
            (255, 0, 0),
            (0, 0, 255),
        )
        .unwrap();

        // Should contain ANSI escape codes
        assert!(result.contains("\x1b["));
        // Should contain styled text
        assert!(result.contains("𝐓"));
    }

    #[test]
    fn test_hue_to_rgb() {
        let red = hue_to_rgb(0.0);
        assert_eq!(red.r, 255);

        let green = hue_to_rgb(120.0);
        assert_eq!(green.g, 255);

        let blue = hue_to_rgb(240.0);
        assert_eq!(blue.b, 255);
    }
}
