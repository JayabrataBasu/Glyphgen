//! Video streaming primitives (frame queue, pacing)

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::sync::Arc;

use image::DynamicImage;

/// A decoded video frame with presentation timestamp
#[derive(Clone)]
pub struct VideoFrame {
    pub image: Arc<DynamicImage>,
    pub pts: Duration,
    pub seq: u64,
}

/// Policy to apply when the frame queue is full
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameDropPolicy {
    /// Drop the oldest frame to make room for the newest
    DropOldest,
    /// Drop the newest incoming frame
    DropNewest,
    /// Block the producer (not yet implemented in this simple queue)
    Block,
}

impl Default for FrameDropPolicy {
    fn default() -> Self {
        FrameDropPolicy::DropOldest
    }
}

/// Bounded frame queue for streaming pipelines
pub struct FrameQueue {
    capacity: usize,
    drop_policy: FrameDropPolicy,
    queue: VecDeque<VideoFrame>,
}

impl FrameQueue {
    pub fn with_capacity(capacity: usize, drop_policy: FrameDropPolicy) -> Self {
        Self {
            capacity: capacity.max(1),
            drop_policy,
            queue: VecDeque::with_capacity(capacity),
        }
    }

    /// Push a frame into the queue. Returns true if enqueued, false if dropped.
    pub fn push(&mut self, frame: VideoFrame) -> bool {
        if self.queue.len() < self.capacity {
            self.queue.push_back(frame);
            return true;
        }

        match self.drop_policy {
            FrameDropPolicy::DropOldest => {
                self.queue.pop_front();
                self.queue.push_back(frame);
                true
            }
            FrameDropPolicy::DropNewest => {
                // Drop the incoming frame
                false
            }
            FrameDropPolicy::Block => {
                // For now, behave like DropNewest to avoid blocking the UI thread
                false
            }
        }
    }

    /// Pop the next frame (FIFO)
    pub fn pop(&mut self) -> Option<VideoFrame> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_drop_policy(&mut self, drop_policy: FrameDropPolicy) {
        self.drop_policy = drop_policy;
    }
}

impl Default for FrameQueue {
    fn default() -> Self {
        Self::with_capacity(120, FrameDropPolicy::default())
    }
}

/// Frame pacing helper for target FPS playback
pub struct FramePacer {
    target_frame: Duration,
    last_frame: Option<Instant>,
}

impl FramePacer {
    pub fn new(target_fps: u32) -> Self {
        let clamped_fps = target_fps.clamp(1, 240);
        let target_frame = Duration::from_secs_f64(1.0 / clamped_fps as f64);
        Self {
            target_frame,
            last_frame: None,
        }
    }

    /// Returns how long to wait until the next frame should be displayed.
    /// If zero, caller can render immediately.
    pub fn time_until_next(&mut self, now: Instant) -> Duration {
        match self.last_frame {
            None => Duration::from_secs(0),
            Some(last) => {
                let elapsed = now.saturating_duration_since(last);
                self.target_frame.saturating_sub(elapsed)
            }
        }
    }

    /// Mark a frame as presented at `now`.
    pub fn mark_presented(&mut self, now: Instant) {
        self.last_frame = Some(now);
    }

    pub fn target_frame(&self) -> Duration {
        self.target_frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_queue_drop_oldest() {
        let img = Arc::new(DynamicImage::new_rgb8(2, 2));
        let mut q = FrameQueue::with_capacity(2, FrameDropPolicy::DropOldest);
        assert!(q.push(VideoFrame { image: img.clone(), pts: Duration::from_millis(0), seq: 1 }));
        assert!(q.push(VideoFrame { image: img.clone(), pts: Duration::from_millis(16), seq: 2 }));
        assert_eq!(q.len(), 2);
        // Third push should drop the oldest
        assert!(q.push(VideoFrame { image: img.clone(), pts: Duration::from_millis(33), seq: 3 }));
        assert_eq!(q.len(), 2);
        let first = q.pop().unwrap();
        assert_eq!(first.seq, 2);
    }

    #[test]
    fn frame_queue_drop_newest() {
        let img = Arc::new(DynamicImage::new_rgb8(2, 2));
        let mut q = FrameQueue::with_capacity(1, FrameDropPolicy::DropNewest);
        assert!(q.push(VideoFrame { image: img.clone(), pts: Duration::from_millis(0), seq: 1 }));
        assert!(!q.push(VideoFrame { image: img.clone(), pts: Duration::from_millis(16), seq: 2 }));
        assert_eq!(q.len(), 1);
        let first = q.pop().unwrap();
        assert_eq!(first.seq, 1);
    }

    #[test]
    fn frame_pacer_basic() {
        let mut pacer = FramePacer::new(60);
        let now = Instant::now();
        assert_eq!(pacer.time_until_next(now), Duration::from_secs(0));
        pacer.mark_presented(now);
        let later = now + Duration::from_millis(5);
        let remaining = pacer.time_until_next(later);
        assert!(remaining > Duration::from_millis(10));
    }
}
