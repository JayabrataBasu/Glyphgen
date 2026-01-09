//! Integration tests for Glyphgen

use glyphgen::render_engines::ascii::{render_ascii, AsciiConfig, CharacterSet};
use glyphgen::render_engines::text_stylizer::{stylize_text, GradientMode, UnicodeStyle};
use glyphgen::render_engines::unicode::{render_unicode, UnicodeConfig, UnicodeMode};
use glyphgen::terminal_capabilities::ColorSupport;
use image::{DynamicImage, RgbImage};

fn create_gradient_image(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let luminance = ((x + y) as f32 / (width + height) as f32 * 255.0) as u8;
            img.put_pixel(x, y, image::Rgb([luminance, luminance, luminance]));
        }
    }
    DynamicImage::ImageRgb8(img)
}

fn create_color_image(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = ((y as f32 / height as f32) * 255.0) as u8;
            let b = 128;
            img.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }
    DynamicImage::ImageRgb8(img)
}

mod ascii_tests {
    use super::*;

    #[test]
    fn test_basic_ascii_render() {
        let image = create_gradient_image(100, 100);
        let config = AsciiConfig {
            target_width: 40,
            charset: CharacterSet::Standard,
            invert: false,
            edge_enhance: false,
        };

        let result = render_ascii(&image, &config).unwrap();

        // Verify basic properties
        assert!(!result.is_empty());
        assert!(result.contains('\n'));

        // Check dimensions
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 0);

        // First line should be target width
        assert_eq!(lines[0].chars().count(), 40);
    }

    #[test]
    fn test_ascii_charsets() {
        let image = create_gradient_image(50, 50);

        for charset in [CharacterSet::Standard, CharacterSet::Extended, CharacterSet::Blocks] {
            let config = AsciiConfig {
                target_width: 20,
                charset: charset.clone(),
                invert: false,
                edge_enhance: false,
            };

            let result = render_ascii(&image, &config).unwrap();
            assert!(!result.is_empty(), "Failed for charset {:?}", charset);
        }
    }

    #[test]
    fn test_ascii_invert() {
        let image = create_gradient_image(50, 50);

        let config_normal = AsciiConfig {
            target_width: 20,
            charset: CharacterSet::Standard,
            invert: false,
            edge_enhance: false,
        };

        let config_inverted = AsciiConfig {
            target_width: 20,
            charset: CharacterSet::Standard,
            invert: true,
            edge_enhance: false,
        };

        let result_normal = render_ascii(&image, &config_normal).unwrap();
        let result_inverted = render_ascii(&image, &config_inverted).unwrap();

        // Results should be different
        assert_ne!(result_normal, result_inverted);
    }

    #[test]
    fn test_ascii_edge_enhance() {
        let image = create_gradient_image(50, 50);

        let config_normal = AsciiConfig {
            target_width: 20,
            charset: CharacterSet::Extended,
            invert: false,
            edge_enhance: false,
        };

        let config_enhanced = AsciiConfig {
            target_width: 20,
            charset: CharacterSet::Extended,
            invert: false,
            edge_enhance: true,
        };

        let result_normal = render_ascii(&image, &config_normal).unwrap();
        let result_enhanced = render_ascii(&image, &config_enhanced).unwrap();

        // Both should succeed
        assert!(!result_normal.is_empty());
        assert!(!result_enhanced.is_empty());
    }

    #[test]
    fn test_ascii_various_widths() {
        let image = create_gradient_image(200, 200);

        for width in [20, 40, 80, 120, 200] {
            let config = AsciiConfig {
                target_width: width,
                charset: CharacterSet::Extended,
                invert: false,
                edge_enhance: false,
            };

            let result = render_ascii(&image, &config).unwrap();
            let first_line = result.lines().next().unwrap();
            assert_eq!(first_line.chars().count(), width, "Width mismatch for {}", width);
        }
    }
}

mod unicode_tests {
    use super::*;

    #[test]
    fn test_basic_unicode_blocks() {
        let image = create_color_image(100, 100);
        let config = UnicodeConfig {
            target_width: 40,
            mode: UnicodeMode::Blocks,
            color_mode: ColorSupport::NoColor,
        };

        let result = render_unicode(&image, &config).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_half_blocks() {
        let image = create_color_image(100, 100);
        let config = UnicodeConfig {
            target_width: 40,
            mode: UnicodeMode::HalfBlocks,
            color_mode: ColorSupport::NoColor,
        };

        let result = render_unicode(&image, &config).unwrap();
        // Should contain upper half block characters
        assert!(result.contains('▀') || result.chars().all(|c| c == ' ' || c == '\n'));
    }

    #[test]
    fn test_braille() {
        let image = create_gradient_image(100, 100);
        let config = UnicodeConfig {
            target_width: 40,
            mode: UnicodeMode::Braille,
            color_mode: ColorSupport::NoColor,
        };

        let result = render_unicode(&image, &config).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_unicode_with_colors() {
        let image = create_color_image(50, 50);

        for color_mode in [
            ColorSupport::NoColor,
            ColorSupport::Color16,
            ColorSupport::Color256,
            ColorSupport::TrueColor,
        ] {
            let config = UnicodeConfig {
                target_width: 20,
                mode: UnicodeMode::HalfBlocks,
                color_mode,
            };

            let result = render_unicode(&image, &config).unwrap();
            assert!(!result.is_empty(), "Failed for color mode {:?}", color_mode);

            // TrueColor should contain ANSI escape codes
            if color_mode == ColorSupport::TrueColor {
                assert!(result.contains("\x1b["));
            }
        }
    }
}

mod text_stylizer_tests {
    use super::*;

    #[test]
    fn test_basic_stylization() {
        let result = stylize_text(
            "Hello",
            UnicodeStyle::Bold,
            GradientMode::None,
            (255, 0, 0),
            (0, 0, 255),
        )
        .unwrap();

        assert!(!result.is_empty());
        assert!(result.contains('𝐇'));
    }

    #[test]
    fn test_all_unicode_styles() {
        let text = "ABC abc 123";

        for style in UnicodeStyle::all() {
            let result = stylize_text(
                text,
                *style,
                GradientMode::None,
                (255, 0, 0),
                (0, 0, 255),
            )
            .unwrap();

            assert!(!result.is_empty(), "Failed for style {:?}", style);
        }
    }

    #[test]
    fn test_gradient_horizontal() {
        let result = stylize_text(
            "Rainbow",
            UnicodeStyle::Bold,
            GradientMode::Horizontal,
            (255, 0, 0),
            (0, 0, 255),
        )
        .unwrap();

        // Should contain ANSI escape codes
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_gradient_rainbow() {
        let result = stylize_text(
            "Rainbow",
            UnicodeStyle::Bold,
            GradientMode::Rainbow,
            (0, 0, 0),
            (0, 0, 0),
        )
        .unwrap();

        // Should contain ANSI escape codes
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_preserves_spaces() {
        let result = stylize_text(
            "A B C",
            UnicodeStyle::Bold,
            GradientMode::None,
            (255, 0, 0),
            (0, 0, 255),
        )
        .unwrap();

        assert!(result.contains(' '));
    }

    #[test]
    fn test_empty_string() {
        let result = stylize_text(
            "",
            UnicodeStyle::Bold,
            GradientMode::None,
            (255, 0, 0),
            (0, 0, 255),
        )
        .unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_special_characters() {
        let result = stylize_text(
            "Hello, World! @#$%",
            UnicodeStyle::Bold,
            GradientMode::None,
            (255, 0, 0),
            (0, 0, 255),
        )
        .unwrap();

        // Punctuation should be preserved
        assert!(result.contains(','));
        assert!(result.contains('!'));
        assert!(result.contains('@'));
    }
}

mod color_space_tests {
    use glyphgen::color_space::*;

    #[test]
    fn test_luminance_black() {
        let lum = rgb_to_luminance(0, 0, 0);
        assert!((lum - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_luminance_white() {
        let lum = rgb_to_luminance(255, 255, 255);
        assert!((lum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_luminance_gray() {
        let lum = rgb_to_luminance(128, 128, 128);
        assert!(lum > 0.1 && lum < 0.9);
    }

    #[test]
    fn test_ansi256_colors() {
        // Pure red
        let red = Rgb::new(255, 0, 0);
        let code = quantize_to_ansi256(red);
        assert!(code >= 16 && code <= 231);

        // Gray
        let gray = Rgb::new(128, 128, 128);
        let gray_code = quantize_to_ansi256(gray);
        // u8 max is 255, so no upper bound check needed
        assert!(gray_code >= 232);
    }

    #[test]
    fn test_color_interpolation() {
        let start = Rgb::new(0, 0, 0);
        let end = Rgb::new(255, 255, 255);

        let mid = interpolate_color(start, end, 0.5);
        assert!((mid.r as i16 - 127).abs() <= 1);
        assert!((mid.g as i16 - 127).abs() <= 1);
        assert!((mid.b as i16 - 127).abs() <= 1);
    }
}

mod unicode_handler_tests {
    use glyphgen::unicode_handler::*;

    #[test]
    fn test_ascii_width() {
        assert_eq!(display_width("Hello"), 5);
        assert_eq!(display_width("World!"), 6);
    }

    #[test]
    fn test_cjk_width() {
        assert_eq!(display_width("你好"), 4);
        assert_eq!(display_width("日本語"), 6);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate_to_width("Hello, World!", 5), "Hello");
        assert_eq!(truncate_to_width("你好世界", 4), "你好");
    }

    #[test]
    fn test_pad() {
        assert_eq!(pad_to_width("Hi", 5, false), "Hi   ");
        assert_eq!(pad_to_width("Hi", 5, true), "   Hi");
    }
}
