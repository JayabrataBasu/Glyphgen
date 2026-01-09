//! Worker thread management
//!
//! Handles background rendering on dedicated worker threads.

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use crossbeam_channel::{unbounded, Receiver, Sender};
use image::DynamicImage;

use crate::render_engines::ascii::{render_ascii, AsciiConfig, CharacterSet};
use crate::render_engines::text_stylizer::{stylize_text, GradientMode, UnicodeStyle};
use crate::render_engines::unicode::{render_unicode, UnicodeConfig, UnicodeMode};
use crate::terminal_capabilities::ColorSupport;

/// Messages sent from main thread to workers
#[derive(Debug)]
pub enum WorkerMessage {
    /// Request ASCII rendering
    AsciiRequest {
        image: Arc<DynamicImage>,
        width: usize,
        charset: CharacterSet,
        invert: bool,
        edge_enhance: bool,
    },
    /// Request Unicode rendering
    UnicodeRequest {
        image: Arc<DynamicImage>,
        width: usize,
        mode: UnicodeMode,
        color_mode: ColorSupport,
    },
    /// Request text stylization
    TextRequest {
        text: String,
        style: UnicodeStyle,
        gradient: GradientMode,
        start_color: (u8, u8, u8),
        end_color: (u8, u8, u8),
    },
    /// Shutdown signal
    Shutdown,
}

/// Responses sent from workers to main thread
#[derive(Debug)]
pub enum WorkerResponse {
    /// ASCII rendering complete
    AsciiComplete { output: String, render_time: u64 },
    /// Unicode rendering complete
    UnicodeComplete { output: String, render_time: u64 },
    /// Text stylization complete
    TextComplete { output: String, render_time: u64 },
    /// Error occurred
    Error(String),
}

/// Handle to worker threads and channels
pub struct WorkerHandle {
    pub request_tx: Sender<WorkerMessage>,
    pub response_rx: Receiver<WorkerResponse>,
    threads: Vec<JoinHandle<()>>,
}

impl WorkerHandle {
    /// Shutdown all worker threads
    pub fn shutdown(self) {
        // Send shutdown signal to all workers
        for _ in &self.threads {
            let _ = self.request_tx.send(WorkerMessage::Shutdown);
        }

        // Wait for threads to finish
        for handle in self.threads {
            let _ = handle.join();
        }
    }
}

/// Spawn worker threads for rendering
pub fn spawn_workers() -> WorkerHandle {
    let (request_tx, request_rx) = unbounded::<WorkerMessage>();
    let (response_tx, response_rx) = unbounded::<WorkerResponse>();

    let mut threads = Vec::new();

    // Spawn multiple worker threads for parallel processing
    let num_workers = num_cpus().min(4).max(1);

    for id in 0..num_workers {
        let rx = request_rx.clone();
        let tx = response_tx.clone();

        let handle = thread::Builder::new()
            .name(format!("render-worker-{}", id))
            .spawn(move || {
                worker_loop(rx, tx);
            })
            .expect("Failed to spawn worker thread");

        threads.push(handle);
    }

    WorkerHandle {
        request_tx,
        response_rx,
        threads,
    }
}

/// Main worker loop - processes messages until shutdown
fn worker_loop(rx: Receiver<WorkerMessage>, tx: Sender<WorkerResponse>) {
    while let Ok(msg) = rx.recv() {
        match msg {
            WorkerMessage::Shutdown => break,

            WorkerMessage::AsciiRequest {
                image,
                width,
                charset,
                invert,
                edge_enhance,
            } => {
                let start = Instant::now();

                let config = AsciiConfig {
                    target_width: width,
                    charset,
                    invert,
                    edge_enhance,
                };

                let response = match render_ascii(&image, &config) {
                    Ok(output) => WorkerResponse::AsciiComplete {
                        output,
                        render_time: start.elapsed().as_millis() as u64,
                    },
                    Err(e) => WorkerResponse::Error(e.to_string()),
                };

                let _ = tx.send(response);
            }

            WorkerMessage::UnicodeRequest {
                image,
                width,
                mode,
                color_mode,
            } => {
                let start = Instant::now();

                let config = UnicodeConfig {
                    target_width: width,
                    mode,
                    color_mode,
                };

                let response = match render_unicode(&image, &config) {
                    Ok(output) => WorkerResponse::UnicodeComplete {
                        output,
                        render_time: start.elapsed().as_millis() as u64,
                    },
                    Err(e) => WorkerResponse::Error(e.to_string()),
                };

                let _ = tx.send(response);
            }

            WorkerMessage::TextRequest {
                text,
                style,
                gradient,
                start_color,
                end_color,
            } => {
                let start = Instant::now();

                let response = match stylize_text(&text, style, gradient, start_color, end_color) {
                    Ok(output) => WorkerResponse::TextComplete {
                        output,
                        render_time: start.elapsed().as_millis() as u64,
                    },
                    Err(e) => WorkerResponse::Error(e.to_string()),
                };

                let _ = tx.send(response);
            }
        }
    }
}

/// Get number of CPUs (fallback to 1)
fn num_cpus() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_spawn_and_shutdown() {
        let workers = spawn_workers();
        workers.shutdown();
    }

    #[test]
    fn test_ascii_request() {
        let workers = spawn_workers();

        let img = Arc::new(DynamicImage::ImageRgb8(RgbImage::new(10, 10)));

        workers
            .request_tx
            .send(WorkerMessage::AsciiRequest {
                image: img,
                width: 10,
                charset: CharacterSet::Standard,
                invert: false,
                edge_enhance: false,
            })
            .unwrap();

        let response = workers
            .response_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();

        match response {
            WorkerResponse::AsciiComplete { output, .. } => {
                assert!(!output.is_empty());
            }
            _ => panic!("Unexpected response type"),
        }

        workers.shutdown();
    }

    #[test]
    fn test_text_request() {
        let workers = spawn_workers();

        workers
            .request_tx
            .send(WorkerMessage::TextRequest {
                text: "Hello".to_string(),
                style: UnicodeStyle::Bold,
                gradient: GradientMode::None,
                start_color: (255, 0, 0),
                end_color: (0, 0, 255),
            })
            .unwrap();

        let response = workers
            .response_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();

        match response {
            WorkerResponse::TextComplete { output, .. } => {
                assert!(!output.is_empty());
                assert!(output.contains('𝐇'));
            }
            _ => panic!("Unexpected response type"),
        }

        workers.shutdown();
    }
}
