//! Performance monitoring
//!
//! Track frame times, FPS, and render times.

use std::collections::VecDeque;
use std::time::Duration;

/// Performance metrics
pub struct PerfMetrics {
    frame_times: VecDeque<Duration>,
    pub fps: f32,
    pub avg_frame_time_ms: f32,
    pub last_render_time_ms: u64,
}

impl PerfMetrics {
    /// Maximum number of frames to track for averaging
    const MAX_SAMPLES: usize = 60;

    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(Self::MAX_SAMPLES),
            fps: 0.0,
            avg_frame_time_ms: 0.0,
            last_render_time_ms: 0,
        }
    }

    /// Record a frame time
    pub fn record_frame(&mut self, duration: Duration) {
        self.frame_times.push_back(duration);
        if self.frame_times.len() > Self::MAX_SAMPLES {
            self.frame_times.pop_front();
        }

        self.update_stats();
    }

    /// Update computed statistics
    fn update_stats(&mut self) {
        if self.frame_times.is_empty() {
            return;
        }

        let total: Duration = self.frame_times.iter().sum();
        let avg = total / self.frame_times.len() as u32;
        self.avg_frame_time_ms = avg.as_secs_f32() * 1000.0;

        if self.avg_frame_time_ms > 0.0 {
            self.fps = 1000.0 / self.avg_frame_time_ms;
        }
    }

    /// Get current FPS as integer
    pub fn fps_int(&self) -> u32 {
        self.fps.round() as u32
    }

    /// Check if performance is degraded (below 30 FPS)
    pub fn is_degraded(&self) -> bool {
        self.fps > 0.0 && self.fps < 30.0
    }
}

impl Default for PerfMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_metrics_initial() {
        let metrics = PerfMetrics::new();
        assert_eq!(metrics.fps, 0.0);
        assert_eq!(metrics.avg_frame_time_ms, 0.0);
    }

    #[test]
    fn test_perf_metrics_recording() {
        let mut metrics = PerfMetrics::new();

        // Record 60 frames at ~16ms each (60 FPS)
        for _ in 0..60 {
            metrics.record_frame(Duration::from_millis(16));
        }

        assert!(metrics.fps > 55.0 && metrics.fps < 65.0);
        assert!(!metrics.is_degraded());
    }

    #[test]
    fn test_perf_metrics_degraded() {
        let mut metrics = PerfMetrics::new();

        // Record slow frames (15 FPS)
        for _ in 0..60 {
            metrics.record_frame(Duration::from_millis(66));
        }

        assert!(metrics.is_degraded());
    }
}
