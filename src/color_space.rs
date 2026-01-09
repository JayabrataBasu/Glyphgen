//! Color space conversion and luminance calculations
//!
//! Handles RGB to grayscale, LAB, and terminal color quantization.

use crate::terminal_capabilities::ColorSupport;

/// RGB color type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_tuple(tuple: (u8, u8, u8)) -> Self {
        Self {
            r: tuple.0,
            g: tuple.1,
            b: tuple.2,
        }
    }

    pub fn to_tuple(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

/// Convert RGB to perceptual luminance (0.0 to 1.0)
///
/// Uses ITU-R BT.709 (HDTV) coefficients for perceptually accurate grayscale.
pub fn rgb_to_luminance(r: u8, g: u8, b: u8) -> f32 {
    0.2126 * (r as f32 / 255.0) + 0.7152 * (g as f32 / 255.0) + 0.0722 * (b as f32 / 255.0)
}

/// Alternative luminance calculation using BT.601 (SDTV) coefficients
pub fn rgb_to_luminance_bt601(r: u8, g: u8, b: u8) -> f32 {
    0.299 * (r as f32 / 255.0) + 0.587 * (g as f32 / 255.0) + 0.114 * (b as f32 / 255.0)
}

/// Convert luminance (0.0 to 1.0) to grayscale byte (0 to 255)
pub fn luminance_to_gray(luminance: f32) -> u8 {
    (luminance.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Quantize RGB to ANSI 256-color palette
pub fn quantize_to_ansi256(rgb: Rgb) -> u8 {
    let r = rgb.r;
    let g = rgb.g;
    let b = rgb.b;

    // Check if it's close to a grayscale value (232-255)
    let avg = ((r as u16 + g as u16 + b as u16) / 3) as u8;
    let gray_diff = (r as i16 - avg as i16).abs().max(
        (g as i16 - avg as i16)
            .abs()
            .max((b as i16 - avg as i16).abs()),
    );

    if gray_diff < 10 {
        // Use grayscale ramp (232-255, 24 levels)
        let gray_index = (avg as f32 / 255.0 * 23.0).round() as u8;
        return 232 + gray_index;
    }

    // Use 6x6x6 color cube (16-231)
    let r_index = (r as f32 / 255.0 * 5.0).round() as u8;
    let g_index = (g as f32 / 255.0 * 5.0).round() as u8;
    let b_index = (b as f32 / 255.0 * 5.0).round() as u8;

    16 + 36 * r_index + 6 * g_index + b_index
}

/// Quantize RGB to ANSI 16-color palette
pub fn quantize_to_ansi16(rgb: Rgb) -> u8 {
    let luminance = rgb_to_luminance(rgb.r, rgb.g, rgb.b);
    let bright = luminance > 0.5;

    // Determine dominant color channel
    let r = rgb.r as f32 / 255.0;
    let g = rgb.g as f32 / 255.0;
    let b = rgb.b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let saturation = if max > 0.0 { (max - min) / max } else { 0.0 };

    // If low saturation, use black/white/gray
    if saturation < 0.2 {
        return if luminance > 0.7 {
            15 // Bright white
        } else if luminance > 0.3 {
            7 // White (light gray)
        } else {
            0 // Black
        };
    }

    // Determine hue-based color
    let color_base = if r >= g && r >= b {
        if g > b {
            3 // Yellow
        } else {
            1 // Red
        }
    } else if g >= r && g >= b {
        if b > r {
            6 // Cyan
        } else {
            2 // Green
        }
    } else {
        if r > g {
            5 // Magenta
        } else {
            4 // Blue
        }
    };

    if bright {
        color_base + 8 // Bright variant
    } else {
        color_base
    }
}

/// Format RGB as ANSI TrueColor escape sequence (foreground)
pub fn rgb_to_ansi_fg(rgb: Rgb) -> String {
    format!("\x1b[38;2;{};{};{}m", rgb.r, rgb.g, rgb.b)
}

/// Format RGB as ANSI TrueColor escape sequence (background)
pub fn rgb_to_ansi_bg(rgb: Rgb) -> String {
    format!("\x1b[48;2;{};{};{}m", rgb.r, rgb.g, rgb.b)
}

/// Format ANSI 256-color escape sequence (foreground)
pub fn ansi256_to_fg(color: u8) -> String {
    format!("\x1b[38;5;{}m", color)
}

/// Format ANSI 256-color escape sequence (background)
pub fn ansi256_to_bg(color: u8) -> String {
    format!("\x1b[48;5;{}m", color)
}

/// Format ANSI 16-color escape sequence (foreground)
pub fn ansi16_to_fg(color: u8) -> String {
    if color < 8 {
        format!("\x1b[{}m", 30 + color)
    } else {
        format!("\x1b[{}m", 90 + (color - 8))
    }
}

/// Format ANSI 16-color escape sequence (background)
pub fn ansi16_to_bg(color: u8) -> String {
    if color < 8 {
        format!("\x1b[{}m", 40 + color)
    } else {
        format!("\x1b[{}m", 100 + (color - 8))
    }
}

/// ANSI reset sequence
pub const ANSI_RESET: &str = "\x1b[0m";

/// Format color for terminal based on color support level
pub fn format_fg_color(rgb: Rgb, support: ColorSupport) -> String {
    match support {
        ColorSupport::NoColor => String::new(),
        ColorSupport::Color16 => ansi16_to_fg(quantize_to_ansi16(rgb)),
        ColorSupport::Color256 => ansi256_to_fg(quantize_to_ansi256(rgb)),
        ColorSupport::TrueColor => rgb_to_ansi_fg(rgb),
    }
}

/// Format background color for terminal based on color support level
pub fn format_bg_color(rgb: Rgb, support: ColorSupport) -> String {
    match support {
        ColorSupport::NoColor => String::new(),
        ColorSupport::Color16 => ansi16_to_bg(quantize_to_ansi16(rgb)),
        ColorSupport::Color256 => ansi256_to_bg(quantize_to_ansi256(rgb)),
        ColorSupport::TrueColor => rgb_to_ansi_bg(rgb),
    }
}

/// Interpolate between two colors
pub fn interpolate_color(start: Rgb, end: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    Rgb {
        r: ((1.0 - t) * start.r as f32 + t * end.r as f32).round() as u8,
        g: ((1.0 - t) * start.g as f32 + t * end.g as f32).round() as u8,
        b: ((1.0 - t) * start.b as f32 + t * end.b as f32).round() as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luminance_range() {
        // Black should be 0
        assert!((rgb_to_luminance(0, 0, 0) - 0.0).abs() < 0.01);

        // White should be 1
        assert!((rgb_to_luminance(255, 255, 255) - 1.0).abs() < 0.01);

        // Gray should be ~0.5
        let gray = rgb_to_luminance(128, 128, 128);
        assert!(gray > 0.2 && gray < 0.8);
    }

    #[test]
    fn test_ansi256_grayscale() {
        let gray = Rgb::new(128, 128, 128);
        let code = quantize_to_ansi256(gray);
        // Should be in grayscale range 232-255
        assert!(code >= 232 && code <= 255);
    }

    #[test]
    fn test_ansi256_color() {
        let red = Rgb::new(255, 0, 0);
        let code = quantize_to_ansi256(red);
        // Should be in color cube range 16-231
        assert!(code >= 16 && code <= 231);
    }

    #[test]
    fn test_color_interpolation() {
        let black = Rgb::new(0, 0, 0);
        let white = Rgb::new(255, 255, 255);

        let mid = interpolate_color(black, white, 0.5);
        assert!((mid.r as i16 - 127).abs() <= 1);
        assert!((mid.g as i16 - 127).abs() <= 1);
        assert!((mid.b as i16 - 127).abs() <= 1);
    }

    #[test]
    fn test_ansi_fg_format() {
        let red = Rgb::new(255, 0, 0);
        let code = rgb_to_ansi_fg(red);
        assert_eq!(code, "\x1b[38;2;255;0;0m");
    }
}
