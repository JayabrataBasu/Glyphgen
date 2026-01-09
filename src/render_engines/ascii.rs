//! ASCII art rendering engine
//!
//! Converts images to ASCII art using luminance-based character mapping.

use anyhow::Result;
use image::{DynamicImage, GenericImageView, GrayImage, Luma};
use serde::{Deserialize, Serialize};

/// Character set for ASCII rendering
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterSet {
    /// Simple 10-character set: " .:-=+*#%@"
    Standard,
    /// Extended 70-character set for more detail
    Extended,
    /// Unicode block elements: " ░▒▓█"
    Blocks,
    /// Custom user-defined character set (sorted dark to light)
    Custom(String),
}

impl Default for CharacterSet {
    fn default() -> Self {
        CharacterSet::Extended
    }
}

impl CharacterSet {
    pub fn name(&self) -> &str {
        match self {
            CharacterSet::Standard => "Standard",
            CharacterSet::Extended => "Extended",
            CharacterSet::Blocks => "Blocks",
            CharacterSet::Custom(_) => "Custom",
        }
    }

    pub fn chars(&self) -> &str {
        match self {
            CharacterSet::Standard => " .:-=+*#%@",
            CharacterSet::Extended => {
                " .'`^\",:;Il!i><~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"
            }
            CharacterSet::Blocks => " ░▒▓█",
            CharacterSet::Custom(chars) => chars,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            CharacterSet::Standard => CharacterSet::Extended,
            CharacterSet::Extended => CharacterSet::Blocks,
            CharacterSet::Blocks => CharacterSet::Standard,
            CharacterSet::Custom(_) => CharacterSet::Standard,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            CharacterSet::Standard => CharacterSet::Blocks,
            CharacterSet::Extended => CharacterSet::Standard,
            CharacterSet::Blocks => CharacterSet::Extended,
            CharacterSet::Custom(_) => CharacterSet::Standard,
        }
    }
}

/// Configuration for ASCII rendering
#[derive(Debug, Clone)]
pub struct AsciiConfig {
    pub target_width: usize,
    pub charset: CharacterSet,
    pub invert: bool,
    pub edge_enhance: bool,
}

impl Default for AsciiConfig {
    fn default() -> Self {
        Self {
            target_width: 80,
            charset: CharacterSet::Extended,
            invert: false,
            edge_enhance: false,
        }
    }
}

/// Render an image as ASCII art
pub fn render_ascii(image: &DynamicImage, config: &AsciiConfig) -> Result<String> {
    // Calculate target dimensions
    // Characters are approximately 2:1 aspect ratio (taller than wide)
    let (width, height) = calculate_dimensions(image, config.target_width);

    // Resize image
    let resized = image.resize_exact(
        width as u32,
        height as u32,
        image::imageops::FilterType::Lanczos3,
    );

    // Convert to grayscale
    let gray = resized.to_luma8();

    // Apply edge enhancement if requested
    let processed = if config.edge_enhance {
        apply_edge_enhancement(&gray)
    } else {
        gray
    };

    // Map pixels to characters
    let charset_chars: Vec<char> = config.charset.chars().chars().collect();
    let num_chars = charset_chars.len();

    let mut output = String::with_capacity((width + 1) * height);

    for y in 0..height {
        for x in 0..width {
            let pixel = processed.get_pixel(x as u32, y as u32);
            let luminance = pixel.0[0] as f32 / 255.0;

            // Optionally invert
            let luminance = if config.invert {
                1.0 - luminance
            } else {
                luminance
            };

            // Map luminance to character index
            let index = (luminance * (num_chars - 1) as f32).round() as usize;
            let index = index.min(num_chars - 1);

            output.push(charset_chars[index]);
        }
        output.push('\n');
    }

    Ok(output)
}

/// Calculate output dimensions maintaining aspect ratio
fn calculate_dimensions(image: &DynamicImage, target_width: usize) -> (usize, usize) {
    let (img_width, img_height) = image.dimensions();
    let aspect_ratio = img_width as f32 / img_height as f32;

    // Terminal characters are roughly 2:1 aspect ratio
    let char_aspect = 0.5;

    let width = target_width;
    let height = ((target_width as f32 / aspect_ratio) * char_aspect).round() as usize;

    // Ensure minimum dimensions
    (width.max(1), height.max(1))
}

/// Apply simple edge enhancement using Sobel operator
fn apply_edge_enhancement(image: &GrayImage) -> GrayImage {
    let (width, height) = image.dimensions();
    let mut output = GrayImage::new(width, height);

    // Sobel kernels
    let sobel_x: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    let sobel_y: [[i32; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let mut gx = 0i32;
            let mut gy = 0i32;

            for ky in 0..3 {
                for kx in 0..3 {
                    let px = image.get_pixel(x + kx - 1, y + ky - 1).0[0] as i32;
                    gx += px * sobel_x[ky as usize][kx as usize];
                    gy += px * sobel_y[ky as usize][kx as usize];
                }
            }

            // Calculate gradient magnitude
            let gradient = ((gx * gx + gy * gy) as f32).sqrt();

            // Blend original with edge (70% original, 30% edge)
            let original = image.get_pixel(x, y).0[0] as f32;
            let enhanced = (original * 0.7 + gradient * 0.3).min(255.0) as u8;

            output.put_pixel(x, y, Luma([enhanced]));
        }
    }

    // Copy border pixels
    for x in 0..width {
        output.put_pixel(x, 0, *image.get_pixel(x, 0));
        output.put_pixel(x, height - 1, *image.get_pixel(x, height - 1));
    }
    for y in 0..height {
        output.put_pixel(0, y, *image.get_pixel(0, y));
        output.put_pixel(width - 1, y, *image.get_pixel(width - 1, y));
    }

    output
}

/// Map a luminance value (0.0-1.0) to a character
pub fn map_luminance_to_char(luminance: f32, charset: &CharacterSet) -> char {
    let chars: Vec<char> = charset.chars().chars().collect();
    let num_chars = chars.len();
    let index = (luminance.clamp(0.0, 1.0) * (num_chars - 1) as f32).round() as usize;
    chars[index.min(num_chars - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charset_cycling() {
        let charset = CharacterSet::Standard;
        assert_eq!(charset.next(), CharacterSet::Extended);
        assert_eq!(charset.prev(), CharacterSet::Blocks);
    }

    #[test]
    fn test_luminance_mapping() {
        let charset = CharacterSet::Standard;

        // Dark (low luminance) should map to first char (space)
        assert_eq!(map_luminance_to_char(0.0, &charset), ' ');

        // Bright (high luminance) should map to last char (@)
        assert_eq!(map_luminance_to_char(1.0, &charset), '@');
    }

    #[test]
    fn test_dimension_calculation() {
        use image::RgbImage;

        // Create a 100x100 test image
        let img = DynamicImage::ImageRgb8(RgbImage::new(100, 100));
        let (width, height) = calculate_dimensions(&img, 80);

        assert_eq!(width, 80);
        // Height should be roughly half of width due to char aspect ratio
        assert!(height >= 30 && height <= 50);
    }

    #[test]
    fn test_render_basic() {
        use image::RgbImage;

        // Create a simple gradient image
        let mut img = RgbImage::new(10, 10);
        for x in 0..10 {
            for y in 0..10 {
                let value = ((x + y) * 12) as u8;
                img.put_pixel(x, y, image::Rgb([value, value, value]));
            }
        }

        let config = AsciiConfig {
            target_width: 10,
            charset: CharacterSet::Standard,
            invert: false,
            edge_enhance: false,
        };

        let result = render_ascii(&DynamicImage::ImageRgb8(img), &config).unwrap();
        assert!(!result.is_empty());
        assert!(result.contains('\n'));
    }
}
