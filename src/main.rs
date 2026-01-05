use clap::Parser as ClapParser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::{
    io::stdout,
    time::{Duration, Instant},
};

use rsvp_term::{
    app::{App, ViewMode},
    orp::calculate_orp,
    parser::{DocumentParser, EpubParser, MarkdownParser},
    timing::calculate_duration,
    types::TimedToken,
    ui,
};

/// Guard struct that ensures terminal cleanup on all exit paths (including panics).
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}

#[derive(ClapParser)]
#[command(name = "rsvp-term")]
#[command(about = "TUI for RSVP reading of markdown and EPUB files")]
struct Cli {
    /// File to read (markdown or EPUB)
    file: std::path::PathBuf,

    /// Export EPUB chapters to markdown files instead of reading
    #[arg(long)]
    export_md: bool,

    /// Maximum width of context lines in characters (prevents reflow on wide terminals)
    #[arg(long, default_value_t = rsvp_term::app::DEFAULT_CONTEXT_WIDTH)]
    context_width: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Validate file exists
    if !cli.file.exists() {
        eprintln!("Error: File not found: {}", cli.file.display());
        std::process::exit(1);
    }

    // Detect file type by extension
    let ext = cli.file.extension().and_then(|e| e.to_str()).unwrap_or("");
    let is_epub = ext.eq_ignore_ascii_case("epub");

    // Handle EPUB export mode
    if cli.export_md {
        if !is_epub {
            eprintln!("Error: --export-md only works with EPUB files");
            std::process::exit(1);
        }
        let parser = EpubParser::new();
        let (book_title, count) = parser.export_chapters(&cli.file)?;
        println!("Exported {} chapters to ./{}/", count, book_title);
        return Ok(());
    }

    // Parse document based on file type
    let doc = if is_epub {
        EpubParser::new().parse_file(&cli.file)?
    } else {
        MarkdownParser::new().parse_file(&cli.file)?
    };

    // Convert to timed tokens
    let wpm = 300u16;
    let timed_tokens: Vec<TimedToken> = doc
        .tokens
        .into_iter()
        .map(|token| {
            let duration = calculate_duration(&token, wpm);
            let orp = calculate_orp(&token.word);
            TimedToken {
                token,
                duration_ms: duration,
                orp_position: orp,
            }
        })
        .collect();

    // Initialize app
    let mut app = App::with_context_width(timed_tokens, doc.sections, cli.context_width);

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let _guard = TerminalGuard; // Cleanup guaranteed on drop
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Main loop
    let mut last_advance = Instant::now();

    loop {
        // Render
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Calculate time until next word using CURRENT wpm (not pre-calculated)
        let next_duration = app
            .current_token()
            .map(|t| Duration::from_millis(calculate_duration(&t.token, app.wpm())))
            .unwrap_or(Duration::from_millis(200));

        // Handle input with timeout
        let timeout = if app.is_paused() || app.view_mode() == ViewMode::Outline {
            Duration::from_millis(100)
        } else {
            let elapsed = last_advance.elapsed();
            next_duration.saturating_sub(elapsed)
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Handle Ctrl+C globally
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        break;
                    }

                    match (app.view_mode(), key.code) {
                        // Global
                        (_, KeyCode::Char('q')) => break,
                        (_, KeyCode::Char('?')) => app.toggle_help(),

                        // Reading mode
                        (ViewMode::Reading, KeyCode::Char(' ')) => app.toggle_pause(),
                        (ViewMode::Reading, KeyCode::Char('j') | KeyCode::Down) => {
                            app.decrease_wpm()
                        }
                        (ViewMode::Reading, KeyCode::Char('k') | KeyCode::Up) => app.increase_wpm(),
                        (ViewMode::Reading, KeyCode::Char('h') | KeyCode::Left) => {
                            app.rewind_sentence()
                        }
                        (ViewMode::Reading, KeyCode::Char('l') | KeyCode::Right) => {
                            app.skip_sentence()
                        }
                        (ViewMode::Reading, KeyCode::Char('o')) => app.toggle_outline(),

                        // Outline mode
                        (ViewMode::Outline, KeyCode::Char('j') | KeyCode::Down) => {
                            app.outline_down()
                        }
                        (ViewMode::Outline, KeyCode::Char('k') | KeyCode::Up) => app.outline_up(),
                        (ViewMode::Outline, KeyCode::Enter) => app.jump_to_section(),
                        (ViewMode::Outline, KeyCode::Esc | KeyCode::Char('o')) => {
                            app.toggle_outline()
                        }

                        _ => {}
                    }
                }
            }
        }

        // Advance word if not paused and in reading mode
        if !app.is_paused()
            && app.view_mode() == ViewMode::Reading
            && last_advance.elapsed() >= next_duration
        {
            app.advance();
            last_advance = Instant::now();
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
