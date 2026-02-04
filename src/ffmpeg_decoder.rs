//! FFmpeg-based video decoder for real video file playback
//!
//! Requires `video` feature to be enabled.

#[cfg(feature = "video")]
use anyhow::Result;
#[cfg(feature = "video")]
use std::path::Path;
#[cfg(feature = "video")]
use std::sync::Arc;
#[cfg(feature = "video")]
use std::time::Duration;

#[cfg(feature = "video")]
use image::DynamicImage;

#[cfg(feature = "video")]
use crate::decoder::FrameSource;

#[cfg(feature = "video")]
use ffmpeg_next as ffmpeg;

/// FFmpeg-powered video decoder
#[cfg(feature = "video")]
pub struct FFmpegDecoder {
    input: ffmpeg::format::context::Input,
    stream_index: usize,
    decoder: ffmpeg::decoder::Video,
    fps: f64,
    total_frames: u64,
    current_frame: u64,
    scaler: ffmpeg::software::scaling::Context,
}

#[cfg(feature = "video")]
impl FFmpegDecoder {
    /// Create a new FFmpeg decoder from a file path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        ffmpeg::init().context("Failed to initialize FFmpeg")?;

        let path = path.as_ref();
        let mut input =
            ffmpeg::format::input(&path).context("Failed to open video file")?;

        let stream_index = input
            .streams()
            .best(ffmpeg::media::Type::Video)
            .context("No video stream found")?
            .index();

        let stream = input.stream(stream_index as _).unwrap();
        let codec = ffmpeg::codec::Context::from_parameters(stream.parameters())
            .context("Failed to create codec context")?;

        let mut decoder = codec.decoder().video().context("Failed to create video decoder")?;

        // Enable hardware acceleration if available
        decoder.set_threading(ffmpeg::threading::Config::count(num_cpus::get() as u32));

        let width = decoder.width() as u32;
        let height = decoder.height() as u32;

        let scaler = decoder
            .scaler(ffmpeg::format::Pixel::RGB24)
            .context("Failed to create scaler")?;

        let fps = stream.avg_frame_rate().unwrap_or((25, 1));
        let fps_val = fps.0 as f64 / fps.1.max(1) as f64;

        let duration = stream.duration() as f64 * stream.time_base().0 as f64 / stream.time_base().1 as f64;
        let total_frames = (duration * fps_val).round() as u64;

        Ok(Self {
            input,
            stream_index,
            decoder,
            fps: fps_val,
            total_frames,
            current_frame: 0,
            scaler,
        })
    }

    fn frame_to_image(&mut self, frame: &ffmpeg::Frame) -> Result<DynamicImage> {
        let video = self
            .scaler
            .run(frame)
            .context("Failed to scale frame")?;

        let width = video.width();
        let height = video.height();
        let data = video.data(0);

        let mut img = RgbImage::new(width, height);
        for (i, pixel) in img.pixels_mut().enumerate() {
            let base = i * 3;
            if base + 2 < data.len() {
                *pixel = image::Rgb([data[base], data[base + 1], data[base + 2]]);
            }
        }

        Ok(DynamicImage::ImageRgb8(img))
    }
}

#[cfg(feature = "video")]
impl FrameSource for FFmpegDecoder {
    fn next_frame(&mut self) -> Result<Option<(Arc<DynamicImage>, Duration)>> {
        let mut packet = ffmpeg::packet::Packet::empty();

        loop {
            match self.input.packets().next() {
                Some((stream, packet_data)) => {
                    if stream.index() != self.stream_index {
                        continue;
                    }

                    packet = packet_data;
                    break;
                }
                None => return Ok(None), // EOF
            }
        }

        self.decoder.send_packet(&packet).ok();

        let mut frame = ffmpeg::frame::Video::empty();
        if self.decoder.receive_frame(&mut frame).is_ok() {
            let image = self.frame_to_image(&frame)?;
            let pts = Duration::from_secs_f64(self.current_frame as f64 / self.fps);
            self.current_frame += 1;
            Ok(Some((Arc::new(image), pts)))
        } else {
            self.next_frame() // Retry on decode delay
        }
    }

    fn duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(self.total_frames as f64 / self.fps))
    }

    fn fps(&self) -> Option<f64> {
        Some(self.fps)
    }

    fn seek(&mut self, timestamp: Duration) -> Result<()> {
        let pts = (timestamp.as_secs_f64() * self.fps) as i64;
        self.decoder
            .send_eof();
        self.input
            .seek(pts, ..)
            .context("Failed to seek")?;
        self.current_frame = pts as u64;
        Ok(())
    }

    fn can_loop(&self) -> bool {
        false // Video files don't loop by default
    }

    fn reset(&mut self) -> Result<()> {
        self.seek(Duration::from_secs(0))
    }
}

/// Stub decoder when video feature is disabled
#[cfg(not(feature = "video"))]
pub struct FFmpegDecoder;

#[cfg(not(feature = "video"))]
impl FFmpegDecoder {
    pub fn new<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Err(anyhow::anyhow!(
            "FFmpeg support not enabled. Compile with `--features video`"
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "video")]
    fn test_ffmpeg_init() {
        // Just test that init doesn't panic
        ffmpeg::init().ok();
    }
}
