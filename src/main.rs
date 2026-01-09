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

    // Initialize terminal
    let mut terminal = setup_terminal()?;

    // Detect terminal capabilities
    let capabilities = detect_capabilities();

    // Load configuration
    let config = Config::load().unwrap_or_default();

    // Spawn worker threads
    let workers = spawn_workers();

    // Create application state
    let mut app_state = AppState::new(config, capabilities, workers.request_tx.clone());

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
