//! Unicode art rendering engine
//!
//! Converts images to Unicode art using block characters with color support.

use anyhow::Result;
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};

use crate::color_space::{format_bg_color, format_fg_color, Rgb, ANSI_RESET};
use crate::terminal_capabilities::ColorSupport;

/// Unicode rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UnicodeMode {
    /// Simple block characters: " ░▒▓█" (4 levels)
    Blocks,
    /// Half-block optimization for 2x vertical resolution
    #[default]
    HalfBlocks,
    /// Braille patterns for 2x4 resolution (experimental)
    Braille,
}

impl UnicodeMode {
    pub fn name(&self) -> &str {
        match self {
            UnicodeMode::Blocks => "Blocks",
            UnicodeMode::HalfBlocks => "Half-Blocks",
            UnicodeMode::Braille => "Braille",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            UnicodeMode::Blocks => UnicodeMode::HalfBlocks,
            UnicodeMode::HalfBlocks => UnicodeMode::Braille,
            UnicodeMode::Braille => UnicodeMode::Blocks,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            UnicodeMode::Blocks => UnicodeMode::Braille,
            UnicodeMode::HalfBlocks => UnicodeMode::Blocks,
            UnicodeMode::Braille => UnicodeMode::HalfBlocks,
        }
    }
}

/// Configuration for Unicode rendering
#[derive(Debug, Clone)]
pub struct UnicodeConfig {
    pub target_width: usize,
    pub mode: UnicodeMode,
    pub color_mode: ColorSupport,
}

impl Default for UnicodeConfig {
    fn default() -> Self {
        Self {
            target_width: 80,
            mode: UnicodeMode::HalfBlocks,
            color_mode: ColorSupport::TrueColor,
        }
    }
}

/// Render an image as Unicode art
pub fn render_unicode(image: &DynamicImage, config: &UnicodeConfig) -> Result<String> {
    match config.mode {
        UnicodeMode::Blocks => render_blocks(image, config),
        UnicodeMode::HalfBlocks => render_half_blocks(image, config),
        UnicodeMode::Braille => render_braille(image, config),
    }
}

/// Render using simple block characters with color
fn render_blocks(image: &DynamicImage, config: &UnicodeConfig) -> Result<String> {
    let (width, height) = calculate_dimensions(image, config.target_width, 1);

    let resized = image.resize_exact(
        width as u32,
        height as u32,
        image::imageops::FilterType::Lanczos3,
    );

    let block_chars = [' ', '░', '▒', '▓', '█'];
    let mut output = String::with_capacity((width * 20 + 1) * height); // Extra space for ANSI codes

    for y in 0..height {
        for x in 0..width {
            let pixel = resized.get_pixel(x as u32, y as u32);
            let rgb = Rgb::new(pixel[0], pixel[1], pixel[2]);

            // Calculate luminance for block selection
            let luminance =
                (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32)
                    / 255.0;
            let block_idx = (luminance * (block_chars.len() - 1) as f32).round() as usize;
            let block_char = block_chars[block_idx.min(block_chars.len() - 1)];

            // Add color if supported
            if config.color_mode != ColorSupport::NoColor {
                output.push_str(&format_fg_color(rgb, config.color_mode));
            }

            output.push(block_char);

            if config.color_mode != ColorSupport::NoColor {
                output.push_str(ANSI_RESET);
            }
        }
        output.push('\n');
    }

    Ok(output)
}

/// Render using half-block characters for 2x vertical resolution
fn render_half_blocks(image: &DynamicImage, config: &UnicodeConfig) -> Result<String> {
    // Double the vertical resolution since each character cell represents 2 rows
    let (width, height) = calculate_dimensions(image, config.target_width, 2);
    let actual_height = height * 2;

    let resized = image.resize_exact(
        width as u32,
        actual_height as u32,
        image::imageops::FilterType::Lanczos3,
    );

    let mut output = String::with_capacity((width * 30 + 1) * height);

    // Process 2 rows at a time
    for y in (0..actual_height).step_by(2) {
        for x in 0..width {
            let top_pixel = resized.get_pixel(x as u32, y as u32);
            let top_rgb = Rgb::new(top_pixel[0], top_pixel[1], top_pixel[2]);

            let bottom_pixel = if y + 1 < actual_height {
                resized.get_pixel(x as u32, (y + 1) as u32)
            } else {
                top_pixel
            };
            let bottom_rgb = Rgb::new(bottom_pixel[0], bottom_pixel[1], bottom_pixel[2]);

            // Use upper half block (▀) with top color as foreground, bottom as background
            if config.color_mode != ColorSupport::NoColor {
                output.push_str(&format_fg_color(top_rgb, config.color_mode));
                output.push_str(&format_bg_color(bottom_rgb, config.color_mode));
            }

            output.push('▀');

            if config.color_mode != ColorSupport::NoColor {
                output.push_str(ANSI_RESET);
            }
        }
        output.push('\n');
    }

    Ok(output)
}

/// Render using Braille patterns for 2x4 resolution
fn render_braille(image: &DynamicImage, config: &UnicodeConfig) -> Result<String> {
    // Braille: each character is 2 wide × 4 tall dots
    let char_width = 2;
    let char_height = 4;

    let output_width = config.target_width;
    let pixel_width = output_width * char_width;

    // Calculate height maintaining aspect ratio
    let (img_width, img_height) = image.dimensions();
    let aspect = img_width as f32 / img_height as f32;
    let pixel_height = (pixel_width as f32 / aspect).round() as usize;
    let output_height = (pixel_height + char_height - 1) / char_height;

    let resized = image.resize_exact(
        pixel_width as u32,
        (output_height * char_height) as u32,
        image::imageops::FilterType::Lanczos3,
    );

    let gray = resized.to_luma8();

    let mut output = String::with_capacity((output_width + 1) * output_height);

    // Braille dot positions (Unicode Braille starts at U+2800)
    // Dot pattern:
    // 1 4
    // 2 5
    // 3 6
    // 7 8
    let dot_values = [
        [0x01, 0x08], // Row 0: dots 1, 4
        [0x02, 0x10], // Row 1: dots 2, 5
        [0x04, 0x20], // Row 2: dots 3, 6
        [0x40, 0x80], // Row 3: dots 7, 8
    ];

    let threshold = 128u8;

    for cy in 0..output_height {
        for cx in 0..output_width {
            let mut braille = 0u8;

            for dy in 0..char_height {
                for dx in 0..char_width {
                    let px = cx * char_width + dx;
                    let py = cy * char_height + dy;

                    if px < pixel_width && py < output_height * char_height {
                        let pixel = gray.get_pixel(px as u32, py as u32);
                        if pixel.0[0] > threshold {
                            braille |= dot_values[dy][dx];
                        }
                    }
                }
            }

            let braille_char = char::from_u32(0x2800 + braille as u32).unwrap_or(' ');
            output.push(braille_char);
        }
        output.push('\n');
    }

    Ok(output)
}

/// Calculate output dimensions
fn calculate_dimensions(
    image: &DynamicImage,
    target_width: usize,
    vertical_multiplier: usize,
) -> (usize, usize) {
    let (img_width, img_height) = image.dimensions();
    let aspect_ratio = img_width as f32 / img_height as f32;

    // Terminal characters are roughly 2:1 aspect ratio
    let char_aspect = 0.5 * vertical_multiplier as f32;

    let width = target_width;
    let height = ((target_width as f32 / aspect_ratio) * char_aspect).round() as usize;

    (width.max(1), height.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_mode_cycling() {
        let mode = UnicodeMode::Blocks;
        assert_eq!(mode.next(), UnicodeMode::HalfBlocks);
        assert_eq!(mode.prev(), UnicodeMode::Braille);
    }

    #[test]
    fn test_render_blocks() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(10, 10));
        let config = UnicodeConfig {
            target_width: 10,
            mode: UnicodeMode::Blocks,
            color_mode: ColorSupport::NoColor,
        };

        let result = render_unicode(&img, &config).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_render_half_blocks() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(10, 10));
        let config = UnicodeConfig {
            target_width: 10,
            mode: UnicodeMode::HalfBlocks,
            color_mode: ColorSupport::NoColor,
        };

        let result = render_unicode(&img, &config).unwrap();
        assert!(!result.is_empty());
        assert!(result.contains('▀') || result.chars().any(|c| c == ' ' || c == '\n'));
    }

    #[test]
    fn test_render_braille() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(10, 10));
        let config = UnicodeConfig {
            target_width: 10,
            mode: UnicodeMode::Braille,
            color_mode: ColorSupport::NoColor,
        };

        let result = render_unicode(&img, &config).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_with_color() {
        let mut img = RgbImage::new(4, 4);
        // Create a simple colored pattern
        for x in 0..4 {
            for y in 0..4 {
                img.put_pixel(x, y, image::Rgb([255, 0, 0])); // Red
            }
        }

        let config = UnicodeConfig {
            target_width: 4,
            mode: UnicodeMode::HalfBlocks,
            color_mode: ColorSupport::TrueColor,
        };

        let result = render_unicode(&DynamicImage::ImageRgb8(img), &config).unwrap();
        // Should contain ANSI escape codes
        assert!(result.contains("\x1b["));
    }
}
