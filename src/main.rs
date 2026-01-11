//! Main entry point for Glyphgen Terminal Art Studio

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    panic,
    time::{Duration, Instant},
};

use glyphgen::{
    config::Config,
    input::handle_event,
    state::AppState,
    terminal_capabilities::detect_capabilities,
    ui,
    worker::{spawn_workers, WorkerHandle},
};

/// Target frame time for 60 FPS
const FRAME_TIME_MS: u64 = 16;

fn main() -> Result<()> {
    // Set up panic hook to restore terminal on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // If --render-once is used we don't want to enter a full TUI (handled below),
    // so we only set up the terminal when needed later.

    // Simple CLI parsing for convenience
    let mut arg_image: Option<std::path::PathBuf> = None;
    let mut arg_render_once = false;
    let mut arg_mode: Option<String> = None;
    let mut arg_output_format: Option<String> = None;

    let mut iter = std::env::args().skip(1);
    while let Some(a) = iter.next() {
        match a.as_str() {
            "--image" => {
                if let Some(p) = iter.next() {
                    arg_image = Some(std::path::PathBuf::from(p));
                }
            }
            "--render-once" => arg_render_once = true,
            "--mode" => {
                if let Some(m) = iter.next() {
                    arg_mode = Some(m);
                }
            }
            "--output-format" => {
                if let Some(f) = iter.next() {
                    arg_output_format = Some(f);
                }
            }
            _ => {}
        }
    }

    // Detect terminal capabilities
    let capabilities = detect_capabilities();

    // Load configuration
    let config = Config::load().unwrap_or_default();

    // Spawn worker threads
    let workers = spawn_workers();

    // If render-once was requested, do not start full TUI — perform a single render + save
    if arg_render_once && arg_image.is_some() {
        return run_render_once(
            arg_image.unwrap(),
            arg_mode.as_deref(),
            arg_output_format.as_deref(),
            &config,
            &workers,
        );
    }

    // Create application state
    let mut app_state = AppState::new(config, capabilities, workers.request_tx.clone());

    // If an image path was provided, set it (this will auto-render)
    if let Some(path) = arg_image {
        match glyphgen::image_loader::load_image(&path) {
            Ok(img) => app_state.set_input_image(path, img),
            Err(e) => eprintln!("Failed to load image: {}", e),
        }
    }

    // Initialize terminal (only needed for interactive TUI)
    let mut terminal = setup_terminal()?;

    // Run main event loop
    let result = run_event_loop(&mut terminal, &mut app_state, &workers);

    // Cleanup
    cleanup_terminal(terminal)?;

    // Shutdown workers
    workers.shutdown();

    result
}

/// Set up the terminal for TUI rendering
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal state
fn cleanup_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Main event loop - handles input, processes worker messages, renders UI
fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app_state: &mut AppState,
    workers: &WorkerHandle,
) -> Result<()> {
    let frame_duration = Duration::from_millis(FRAME_TIME_MS);

    loop {
        let frame_start = Instant::now();

        // Render UI
        terminal.draw(|frame| ui::render(frame, app_state))?;

        // Poll for events with timeout
        let timeout = frame_duration.saturating_sub(frame_start.elapsed());
        if event::poll(timeout)? {
            let event = event::read()?;

            // Handle terminal resize
            if let Event::Resize(width, height) = event {
                app_state.set_terminal_size(width, height);
            }

            // Handle input
            handle_event(event, app_state)?;
        }

        // Process worker responses (non-blocking)
        while let Ok(response) = workers.response_rx.try_recv() {
            app_state.handle_worker_response(response);
        }

        // Record frame time for performance monitoring
        let frame_time = frame_start.elapsed();
        app_state.perf_metrics.record_frame(frame_time);

        // Check for quit
        if app_state.should_quit {
            break;
        }
    }

    // Save configuration on exit
    if let Err(e) = app_state.config.save() {
        eprintln!("Warning: Failed to save config: {}", e);
    }

    Ok(())
}

/// Render once: load image, request worker, wait for response, save output to file
fn run_render_once(
    image_path: std::path::PathBuf,
    mode: Option<&str>,
    output_format: Option<&str>,
    config: &Config,
    workers: &WorkerHandle,
) -> Result<()> {
    let img = glyphgen::image_loader::load_image(&image_path)?;

    // Determine mode
    let mode_enum = match mode.unwrap_or("ascii") {
        "ascii" => glyphgen::state::RenderMode::ImageToAscii,
        "unicode" => glyphgen::state::RenderMode::ImageToUnicode,
        "text" => glyphgen::state::RenderMode::TextStylizer,
        other => {
            eprintln!("Unknown mode '{}', defaulting to ascii", other);
            glyphgen::state::RenderMode::ImageToAscii
        }
    };

    // Construct message based on mode
    use glyphgen::worker::WorkerMessage;
    let config_clone = config.clone();

    match mode_enum {
        glyphgen::state::RenderMode::ImageToAscii => {
            let msg = WorkerMessage::AsciiRequest {
                image: std::sync::Arc::new(img),
                width: config_clone.ascii.default_width,
                charset: glyphgen::render_engines::ascii::CharacterSet::Extended,
                invert: false,
                edge_enhance: config_clone.ascii.edge_enhance,
            };
            let _ = workers.request_tx.send(msg);
        }
        glyphgen::state::RenderMode::ImageToUnicode => {
            let msg = WorkerMessage::UnicodeRequest {
                image: std::sync::Arc::new(img),
                width: config_clone.unicode.default_width,
                mode: glyphgen::render_engines::unicode::UnicodeMode::HalfBlocks,
                color_mode: glyphgen::terminal_capabilities::ColorSupport::TrueColor,
            };
            let _ = workers.request_tx.send(msg);
        }
        glyphgen::state::RenderMode::TextStylizer => {
            let msg = WorkerMessage::TextRequest {
                text: String::from("Example Text"),
                style: glyphgen::render_engines::text_stylizer::UnicodeStyle::Bold,
                gradient: glyphgen::render_engines::text_stylizer::GradientMode::None,
                start_color: (255, 0, 0),
                end_color: (0, 0, 255),
            };
            let _ = workers.request_tx.send(msg);
        }
    }

    // Wait for result
    use std::time::Duration;
    if let Ok(response) = workers.response_rx.recv_timeout(Duration::from_secs(10)) {
        match response {
            glyphgen::worker::WorkerResponse::AsciiComplete { output, render_time } => {
                // ASCII mode: save based on format
                let (out_file, content) = match output_format.unwrap_or("txt") {
                    "html" => (
                        "ascii_output.html",
                        glyphgen::input::convert_ansi_to_html(&output),
                    ),
                    "ansi" => ("ascii_output.ansi", output),
                    _ => ("ascii_output.txt", output),
                };
                std::fs::write(out_file, content)?;
                println!("Saved ASCII output to {} ({}ms)", out_file, render_time);
            }
            glyphgen::worker::WorkerResponse::UnicodeComplete { output, render_time } => {
                // Unicode mode: ANSI/HTML/PNG/SVG only (no TXT)
                let (out_file, content) = match output_format.unwrap_or("ansi") {
                    "html" => (
                        "unicode_output.html",
                        glyphgen::input::convert_ansi_to_html(&output),
                    ),
                    "png" => {
                        glyphgen::input::export_to_png(&output, "unicode_output.png")?;
                        println!("Saved Unicode PNG to unicode_output.png ({}ms)", render_time);
                        return Ok(());
                    }
                    "svg" => {
                        glyphgen::input::export_to_svg(&output, "unicode_output.svg")?;
                        println!("Saved Unicode SVG to unicode_output.svg ({}ms)", render_time);
                        return Ok(());
                    }
                    _ => ("unicode_output.ansi", output),
                };
                std::fs::write(out_file, content)?;
                println!("Saved Unicode output to {} ({}ms)", out_file, render_time);
            }
            glyphgen::worker::WorkerResponse::TextComplete { output, render_time } => {
                // Text stylizer: save based on format
                let (out_file, content) = match output_format.unwrap_or("txt") {
                    "html" => (
                        "styled_text.html",
                        glyphgen::input::convert_ansi_to_html(&output),
                    ),
                    "ansi" => ("styled_text.ansi", output),
                    _ => ("styled_text.txt", output),
                };
                std::fs::write(out_file, content)?;
                println!("Saved text output to {} ({}ms)", out_file, render_time);
            }
            glyphgen::worker::WorkerResponse::Error(err) => {
                eprintln!("Render error: {}", err);
            }
        }
    } else {
        eprintln!("Timed out waiting for render response");
    }

    Ok(())
}
