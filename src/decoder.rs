//! Video decoding abstractions
//!
//! Provides trait-based decoder interface for various frame sources.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use image::DynamicImage;

/// Frame source abstraction
pub trait FrameSource: Send {
    /// Get the next frame with its presentation timestamp.
    /// Returns None when the source is exhausted.
    fn next_frame(&mut self) -> Result<Option<(Arc<DynamicImage>, Duration)>>;

    /// Get the total duration if known
    fn duration(&self) -> Option<Duration>;

    /// Get the native frame rate if known
    fn fps(&self) -> Option<f64>;

    /// Seek to a specific timestamp (optional, not all sources support this)
    fn seek(&mut self, _timestamp: Duration) -> Result<()> {
        Ok(())
    }

    /// Check if the source can loop/repeat
    fn can_loop(&self) -> bool {
        false
    }

    /// Reset to beginning (for looping sources)
    fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Frame source that repeats a single image at a fixed rate
pub struct StaticImageSource {
    image: Arc<DynamicImage>,
    fps: f64,
    frame_count: u64,
    max_frames: Option<u64>,
}

impl StaticImageSource {
    pub fn new(image: Arc<DynamicImage>, fps: f64) -> Self {
        Self {
            image,
            fps,
            frame_count: 0,
            max_frames: None,
        }
    }

    pub fn with_max_frames(mut self, max: u64) -> Self {
        self.max_frames = Some(max);
        self
    }
}

impl FrameSource for StaticImageSource {
    fn next_frame(&mut self) -> Result<Option<(Arc<DynamicImage>, Duration)>> {
        if let Some(max) = self.max_frames {
            if self.frame_count >= max {
                return Ok(None);
            }
        }

        let pts = Duration::from_secs_f64(self.frame_count as f64 / self.fps);
        self.frame_count += 1;
        Ok(Some((Arc::clone(&self.image), pts)))
    }

    fn duration(&self) -> Option<Duration> {
        self.max_frames
            .map(|max| Duration::from_secs_f64(max as f64 / self.fps))
    }

    fn fps(&self) -> Option<f64> {
        Some(self.fps)
    }

    fn can_loop(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<()> {
        self.frame_count = 0;
        Ok(())
    }
}

/// Frame source that cycles through a sequence of image files
pub struct ImageSequenceSource {
    paths: Vec<PathBuf>,
    current_index: usize,
    fps: f64,
    loop_mode: bool,
    frame_count: u64,
}

impl ImageSequenceSource {
    pub fn new(paths: Vec<PathBuf>, fps: f64) -> Self {
        Self {
            paths,
            current_index: 0,
            fps,
            loop_mode: true,
            frame_count: 0,
        }
    }

    pub fn with_loop(mut self, loop_mode: bool) -> Self {
        self.loop_mode = loop_mode;
        self
    }
}

impl FrameSource for ImageSequenceSource {
    fn next_frame(&mut self) -> Result<Option<(Arc<DynamicImage>, Duration)>> {
        if self.paths.is_empty() {
            return Ok(None);
        }

        if self.current_index >= self.paths.len() {
            if self.loop_mode {
                self.current_index = 0;
            } else {
                return Ok(None);
            }
        }

        let path = &self.paths[self.current_index];
        let img = image::open(path)
            .with_context(|| format!("Failed to load frame: {}", path.display()))?;

        let pts = Duration::from_secs_f64(self.frame_count as f64 / self.fps);
        self.current_index += 1;
        self.frame_count += 1;

        Ok(Some((Arc::new(img), pts)))
    }

    fn duration(&self) -> Option<Duration> {
        if self.loop_mode {
            None
        } else {
            Some(Duration::from_secs_f64(self.paths.len() as f64 / self.fps))
        }
    }

    fn fps(&self) -> Option<f64> {
        Some(self.fps)
    }

    fn can_loop(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<()> {
        self.current_index = 0;
        self.frame_count = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_static_image_source() {
        let img = Arc::new(DynamicImage::ImageRgb8(RgbImage::new(10, 10)));
        let mut source = StaticImageSource::new(img, 30.0).with_max_frames(5);

        assert_eq!(source.fps(), Some(30.0));
        assert!(source.can_loop());

        for i in 0..5 {
            let frame = source.next_frame().unwrap();
            assert!(frame.is_some(), "Frame {} should exist", i);
        }

        let frame = source.next_frame().unwrap();
        assert!(frame.is_none(), "Should be exhausted after max frames");

        source.reset().unwrap();
        let frame = source.next_frame().unwrap();
        assert!(frame.is_some(), "Should have frames after reset");
    }
}
