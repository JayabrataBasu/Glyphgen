//! Terminal capability detection
//!
//! Detects color support, Unicode support, and terminal size.

use crossterm::terminal;
use std::env;

/// Level of color support in the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorSupport {
    /// No color support
    NoColor,
    /// 16 basic colors
    Color16,
    /// 256 color palette
    Color256,
    /// Full 24-bit RGB (TrueColor)
    #[default]
    TrueColor,
}

impl ColorSupport {
    pub fn name(&self) -> &'static str {
        match self {
            ColorSupport::NoColor => "None",
            ColorSupport::Color16 => "16 Colors",
            ColorSupport::Color256 => "256 Colors",
            ColorSupport::TrueColor => "True Color",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ColorSupport::NoColor => ColorSupport::Color16,
            ColorSupport::Color16 => ColorSupport::Color256,
            ColorSupport::Color256 => ColorSupport::TrueColor,
            ColorSupport::TrueColor => ColorSupport::NoColor,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ColorSupport::NoColor => ColorSupport::TrueColor,
            ColorSupport::Color16 => ColorSupport::NoColor,
            ColorSupport::Color256 => ColorSupport::Color16,
            ColorSupport::TrueColor => ColorSupport::Color256,
        }
    }
}

/// Level of Unicode support
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeSupport {
    /// ASCII only
    Ascii,
    /// Basic Unicode (Latin, symbols)
    Basic,
    /// Full Unicode (including CJK, emoji, mathematical symbols)
    Full,
}

/// Terminal capabilities
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub color_support: ColorSupport,
    pub unicode_support: UnicodeSupport,
    pub mouse_support: bool,
    pub size: (u16, u16),
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self {
            color_support: ColorSupport::TrueColor,
            unicode_support: UnicodeSupport::Full,
            mouse_support: true,
            size: (80, 24),
        }
    }
}

/// Detect terminal capabilities
pub fn detect_capabilities() -> TerminalCapabilities {
    let color_support = detect_color_support();
    let unicode_support = detect_unicode_support();
    let size = terminal::size().unwrap_or((80, 24));

    TerminalCapabilities {
        color_support,
        unicode_support,
        mouse_support: true, // Crossterm always supports mouse
        size,
    }
}

/// Detect the level of color support
fn detect_color_support() -> ColorSupport {
    // Check NO_COLOR environment variable (standard for disabling colors)
    if env::var("NO_COLOR").is_ok() {
        return ColorSupport::NoColor;
    }

    // Check COLORTERM for TrueColor support
    if let Ok(colorterm) = env::var("COLORTERM") {
        let colorterm = colorterm.to_lowercase();
        if colorterm.contains("truecolor") || colorterm.contains("24bit") {
            return ColorSupport::TrueColor;
        }
    }

    // Check TERM for color hints
    if let Ok(term) = env::var("TERM") {
        let term = term.to_lowercase();

        // Modern terminals with TrueColor
        if term.contains("kitty")
            || term.contains("alacritty")
            || term.contains("iterm")
            || term.contains("vte")
            || term.contains("256color")
        {
            // Many 256color terminals also support TrueColor
            if env::var("COLORTERM").is_ok() {
                return ColorSupport::TrueColor;
            }
            return ColorSupport::Color256;
        }

        // Check for xterm variants
        if term.contains("xterm") {
            if term.contains("256") {
                return ColorSupport::Color256;
            }
            return ColorSupport::Color16;
        }

        // Check for screen/tmux
        if term.contains("screen") || term.contains("tmux") {
            return ColorSupport::Color256;
        }

        // Basic terminal
        if term.contains("linux") || term.contains("console") {
            return ColorSupport::Color16;
        }
    }

    // Windows Terminal detection
    if env::var("WT_SESSION").is_ok() {
        return ColorSupport::TrueColor;
    }

    // Default to 256 colors if nothing detected (most modern terminals)
    ColorSupport::Color256
}

/// Detect Unicode support level
fn detect_unicode_support() -> UnicodeSupport {
    // Check LANG and LC_ALL for UTF-8
    let has_utf8 = env::var("LANG")
        .or_else(|_| env::var("LC_ALL"))
        .or_else(|_| env::var("LC_CTYPE"))
        .map(|v| v.to_uppercase().contains("UTF"))
        .unwrap_or(false);

    if !has_utf8 {
        return UnicodeSupport::Ascii;
    }

    // Check terminal type for full Unicode support
    if let Ok(term) = env::var("TERM") {
        let term = term.to_lowercase();

        // Known good terminals for full Unicode
        if term.contains("kitty")
            || term.contains("alacritty")
            || term.contains("iterm")
            || term.contains("vte")
        {
            return UnicodeSupport::Full;
        }
    }

    // Windows Terminal has full Unicode support
    if env::var("WT_SESSION").is_ok() {
        return UnicodeSupport::Full;
    }

    // Default to full if UTF-8 locale is set
    if has_utf8 {
        UnicodeSupport::Full
    } else {
        UnicodeSupport::Basic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_support_cycling() {
        let support = ColorSupport::NoColor;
        assert_eq!(support.next(), ColorSupport::Color16);
        assert_eq!(support.prev(), ColorSupport::TrueColor);
    }

    #[test]
    fn test_capabilities_default() {
        let caps = TerminalCapabilities::default();
        assert_eq!(caps.color_support, ColorSupport::TrueColor);
        assert_eq!(caps.unicode_support, UnicodeSupport::Full);
        assert!(caps.mouse_support);
    }
}
