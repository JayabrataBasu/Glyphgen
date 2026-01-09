//! Image loading utilities
//!
//! Handles async loading and decoding of images.

use anyhow::{Context, Result};
use image::DynamicImage;
use std::path::Path;

/// Load an image from a file path
///
/// Supports PNG, JPEG, GIF, BMP, and WebP formats.
pub fn load_image(path: &Path) -> Result<DynamicImage> {
    let img = image::open(path).with_context(|| format!("Failed to load image: {:?}", path))?;
    Ok(img)
}

/// Load an image from bytes
pub fn load_image_from_bytes(bytes: &[u8]) -> Result<DynamicImage> {
    let img = image::load_from_memory(bytes).context("Failed to decode image from memory")?;
    Ok(img)
}

/// Get supported image format extensions
pub fn supported_extensions() -> &'static [&'static str] {
    &["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif"]
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
        // Create a minimal valid PNG (1x1 white pixel)
        let png_data: [u8; 67] = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // 8-bit RGB
            0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
            0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF, 0xFF, // data
            0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59, // checksum
            0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
            0x44, 0xAE, 0x42, 0x60, 0x82,
        ];

        let result = load_image_from_bytes(&png_data);
        assert!(result.is_ok());
    }
}
