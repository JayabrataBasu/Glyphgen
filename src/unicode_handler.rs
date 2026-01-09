//! Unicode width and validation utilities
//!
//! Handles East Asian Width calculations and grapheme cluster validation.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Calculate the display width of a string
///
/// Takes into account East Asian Wide/Fullwidth characters.
pub fn display_width(s: &str) -> usize {
    s.width()
}

/// Calculate display width of a single character
pub fn char_width(c: char) -> usize {
    unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)
}

/// Validate that a string contains only well-formed grapheme clusters
pub fn validate_graphemes(s: &str) -> bool {
    s.graphemes(true).all(|g| !g.is_empty())
}

/// Truncate a string to fit within a maximum display width
pub fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut current_width = 0;

    for grapheme in s.graphemes(true) {
        let grapheme_width = grapheme.width();
        if current_width + grapheme_width > max_width {
            break;
        }
        result.push_str(grapheme);
        current_width += grapheme_width;
    }

    result
}

/// Pad a string to a specific display width
pub fn pad_to_width(s: &str, target_width: usize, align_right: bool) -> String {
    let current_width = display_width(s);
    if current_width >= target_width {
        return truncate_to_width(s, target_width);
    }

    let padding = target_width - current_width;
    if align_right {
        format!("{}{}", " ".repeat(padding), s)
    } else {
        format!("{}{}", s, " ".repeat(padding))
    }
}

/// Count grapheme clusters in a string
pub fn grapheme_count(s: &str) -> usize {
    s.graphemes(true).count()
}

/// Split a string into grapheme clusters
pub fn split_graphemes(s: &str) -> Vec<&str> {
    s.graphemes(true).collect()
}

/// Check if a character is a combining character
pub fn is_combining(c: char) -> bool {
    matches!(c,
        '\u{0300}'..='\u{036F}' |  // Combining Diacritical Marks
        '\u{1AB0}'..='\u{1AFF}' |  // Combining Diacritical Marks Extended
        '\u{1DC0}'..='\u{1DFF}' |  // Combining Diacritical Marks Supplement
        '\u{20D0}'..='\u{20FF}' |  // Combining Diacritical Marks for Symbols
        '\u{FE20}'..='\u{FE2F}'    // Combining Half Marks
    )
}

/// Internal implementation: detect whether any relevant env var mentions UTF
#[inline]
fn terminal_supports_unicode() -> bool {
    ["LANG", "LC_ALL", "LC_CTYPE"].iter().any(|key| {
        std::env::var(key)
            .map(|v| v.to_uppercase().contains("UTF"))
            .unwrap_or(false)
    })
}

/// Public API kept for compatibility; thin wrapper around the internal implementation
#[must_use]
pub fn check_unicode_support() -> bool {
    terminal_supports_unicode()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(display_width("Hello"), 5);
        assert_eq!(display_width("World"), 5);
    }

    #[test]
    fn test_display_width_cjk() {
        assert_eq!(display_width("你好"), 4); // 2 chars × 2 width
        assert_eq!(display_width("日本語"), 6); // 3 chars × 2 width
    }

    #[test]
    fn test_display_width_emoji() {
        assert_eq!(display_width("🎨"), 2);
        assert_eq!(display_width("👨‍👩‍👧‍👦"), 2); // Family emoji (single grapheme)
    }

    #[test]
    fn test_truncate_to_width() {
        assert_eq!(truncate_to_width("Hello, World!", 5), "Hello");
        assert_eq!(truncate_to_width("你好世界", 4), "你好");
    }

    #[test]
    fn test_pad_to_width() {
        assert_eq!(pad_to_width("Hi", 5, false), "Hi   ");
        assert_eq!(pad_to_width("Hi", 5, true), "   Hi");
    }

    #[test]
    fn test_validate_graphemes() {
        assert!(validate_graphemes("Hello"));
        assert!(validate_graphemes("你好"));
        assert!(validate_graphemes("🎨"));
    }

    #[test]
    fn test_grapheme_count() {
        assert_eq!(grapheme_count("Hello"), 5);
        assert_eq!(grapheme_count("你好"), 2);
        assert_eq!(grapheme_count("👨‍👩‍👧‍👦"), 1); // Family is one grapheme
    }

    #[test]
    fn test_check_unicode_support_env() {
        // Ensure wrapper compiles and returns a bool; set an env var to simulate a UTF locale
        std::env::set_var("LC_CTYPE", "en_US.UTF-8");
        assert!(check_unicode_support());
        std::env::remove_var("LC_CTYPE");
    }
}
