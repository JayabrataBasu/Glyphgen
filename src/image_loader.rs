//! Image loading utilities
//!
//! Handles async loading and decoding of images and video files.

use anyhow::{Context, Result};
use image::DynamicImage;
use std::path::Path;

use crate::decoder::FrameSource;

/// Load an image from a file path
///
/// Supports PNG, JPEG, GIF, BMP, and WebP formats.
pub fn load_image(path: &Path) -> Result<DynamicImage> {
    let img = image::open(path).with_context(|| format!("Failed to load image: {:?}", path))?;
    Ok(img)
}

/// Load a video or create an appropriate frame source
///
/// Detects format:
/// - Video files (MP4, WebM, MKV, etc.) → FFmpegDecoder if feature enabled
/// - Image sequences (numbered files) → ImageSequenceSource
/// - Single images → StaticImageSource
pub fn load_video_or_image(path: &Path) -> Result<Box<dyn FrameSource>> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check if it's a video format
    let video_exts = ["mp4", "mkv", "webm", "avi", "mov", "flv", "wmv", "m4v"];
    if video_exts.contains(&ext.as_str()) {
        #[cfg(feature = "video")]
        {
            use crate::ffmpeg_decoder::FFmpegDecoder;
            let decoder = FFmpegDecoder::new(path)?;
            return Ok(Box::new(decoder));
        }
        #[cfg(not(feature = "video"))]
        {
            return Err(anyhow::anyhow!(
                "Video support not enabled. Compile with `--features video` to play {}",
                ext
            ));
        }
    }

    // Otherwise treat as image (will create StaticImageSource)
    let img = load_image(path)?;
    use crate::decoder::StaticImageSource;
    use std::sync::Arc;
    Ok(Box::new(StaticImageSource::new(Arc::new(img), 30.0)))
}

/// Load an image from bytes
pub fn load_image_from_bytes(bytes: &[u8]) -> Result<DynamicImage> {
    let img = image::load_from_memory(bytes).context("Failed to decode image from memory")?;
    Ok(img)
}

/// Supported image format extensions
static SUPPORTED_EXTENSIONS_ARRAY: [&'static str; 8] = ["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif"];

/// Get supported image format extensions
pub fn supported_extensions() -> &'static [&'static str] {
    &SUPPORTED_EXTENSIONS_ARRAY[..]
}

/// Check if a file extension is a supported image format
pub fn is_supported_format(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            supported_extensions().iter().any(|&e| e == ext_lower)
        })
        .unwrap_or(false)
}

/// Get image dimensions
pub fn get_image_dimensions(path: &Path) -> Result<(u32, u32)> {
    let reader = image::ImageReader::open(path)
        .with_context(|| format!("Failed to open image: {:?}", path))?;
    let dimensions = reader.into_dimensions()?;
    Ok(dimensions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_supported_extensions() {
        let extensions = supported_extensions();
        assert!(extensions.contains(&"png"));
        assert!(extensions.contains(&"jpg"));
        assert!(extensions.contains(&"jpeg"));
    }

    #[test]
    fn test_is_supported_format() {
        assert!(is_supported_format(&PathBuf::from("test.png")));
        assert!(is_supported_format(&PathBuf::from("test.PNG")));
        assert!(is_supported_format(&PathBuf::from("test.jpg")));
        assert!(is_supported_format(&PathBuf::from("test.JPEG")));
        assert!(!is_supported_format(&PathBuf::from("test.txt")));
        assert!(!is_supported_format(&PathBuf::from("test")));
    }

    #[test]
    fn test_load_from_bytes() {
        // Test with a simple test - just verify the function exists and handles errors
        // Invalid data should return an error
        let invalid_data: &[u8] = &[0u8; 10][..];
        let result = load_image_from_bytes(invalid_data);
        assert!(result.is_err());
    }
}
