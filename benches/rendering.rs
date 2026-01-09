//! Rendering benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glyphgen::render_engines::ascii::{render_ascii, AsciiConfig, CharacterSet};
use glyphgen::render_engines::text_stylizer::{stylize_text, GradientMode, UnicodeStyle};
use glyphgen::render_engines::unicode::{render_unicode, UnicodeConfig, UnicodeMode};
use glyphgen::terminal_capabilities::ColorSupport;
use image::{DynamicImage, RgbImage};

fn create_test_image(width: u32, height: u32) -> DynamicImage {
    let mut img = RgbImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = ((y as f32 / height as f32) * 255.0) as u8;
            let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
            img.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }
    DynamicImage::ImageRgb8(img)
}

fn benchmark_ascii_render(c: &mut Criterion) {
    let image = create_test_image(800, 600);

    let mut group = c.benchmark_group("ASCII Rendering");

    for width in [40, 80, 120, 160].iter() {
        let config = AsciiConfig {
            target_width: *width,
            charset: CharacterSet::Extended,
            invert: false,
            edge_enhance: false,
        };

        group.bench_function(format!("width_{}", width), |b| {
            b.iter(|| render_ascii(black_box(&image), black_box(&config)))
        });
    }

    group.finish();
}

fn benchmark_ascii_with_edge_enhance(c: &mut Criterion) {
    let image = create_test_image(800, 600);

    let config_without = AsciiConfig {
        target_width: 80,
        charset: CharacterSet::Extended,
        invert: false,
        edge_enhance: false,
    };

    let config_with = AsciiConfig {
        target_width: 80,
        charset: CharacterSet::Extended,
        invert: false,
        edge_enhance: true,
    };

    let mut group = c.benchmark_group("ASCII Edge Enhancement");

    group.bench_function("without_edge", |b| {
        b.iter(|| render_ascii(black_box(&image), black_box(&config_without)))
    });

    group.bench_function("with_edge", |b| {
        b.iter(|| render_ascii(black_box(&image), black_box(&config_with)))
    });

    group.finish();
}

fn benchmark_unicode_render(c: &mut Criterion) {
    let image = create_test_image(800, 600);

    let mut group = c.benchmark_group("Unicode Rendering");

    for mode in [UnicodeMode::Blocks, UnicodeMode::HalfBlocks, UnicodeMode::Braille].iter() {
        let config = UnicodeConfig {
            target_width: 80,
            mode: *mode,
            color_mode: ColorSupport::NoColor,
        };

        group.bench_function(format!("{:?}", mode), |b| {
            b.iter(|| render_unicode(black_box(&image), black_box(&config)))
        });
    }

    group.finish();
}

fn benchmark_unicode_color_modes(c: &mut Criterion) {
    let image = create_test_image(400, 300);

    let mut group = c.benchmark_group("Unicode Color Modes");

    for color_mode in [
        ColorSupport::NoColor,
        ColorSupport::Color16,
        ColorSupport::Color256,
        ColorSupport::TrueColor,
    ]
    .iter()
    {
        let config = UnicodeConfig {
            target_width: 80,
            mode: UnicodeMode::HalfBlocks,
            color_mode: *color_mode,
        };

        group.bench_function(format!("{:?}", color_mode), |b| {
            b.iter(|| render_unicode(black_box(&image), black_box(&config)))
        });
    }

    group.finish();
}

fn benchmark_text_stylizer(c: &mut Criterion) {
    let test_text = "The quick brown fox jumps over the lazy dog. 1234567890";

    let mut group = c.benchmark_group("Text Stylizer");

    for style in [
        UnicodeStyle::Bold,
        UnicodeStyle::Italic,
        UnicodeStyle::Script,
        UnicodeStyle::DoubleStruck,
    ]
    .iter()
    {
        group.bench_function(format!("{:?}", style), |b| {
            b.iter(|| {
                stylize_text(
                    black_box(test_text),
                    black_box(*style),
                    black_box(GradientMode::None),
                    black_box((255, 0, 0)),
                    black_box((0, 0, 255)),
                )
            })
        });
    }

    group.finish();
}

fn benchmark_text_gradients(c: &mut Criterion) {
    let test_text = "The quick brown fox jumps over the lazy dog. 1234567890";

    let mut group = c.benchmark_group("Text Gradients");

    for gradient in [GradientMode::None, GradientMode::Horizontal, GradientMode::Rainbow].iter() {
        group.bench_function(format!("{:?}", gradient), |b| {
            b.iter(|| {
                stylize_text(
                    black_box(test_text),
                    black_box(UnicodeStyle::Bold),
                    black_box(*gradient),
                    black_box((255, 0, 0)),
                    black_box((0, 0, 255)),
                )
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_ascii_render,
    benchmark_ascii_with_edge_enhance,
    benchmark_unicode_render,
    benchmark_unicode_color_modes,
    benchmark_text_stylizer,
    benchmark_text_gradients,
);

criterion_main!(benches);
