use rsvp_term::app::App;
use rsvp_term::orp::calculate_orp;
use rsvp_term::parser::{DocumentParser, MarkdownParser};
use rsvp_term::timing::calculate_duration;
use rsvp_term::types::TimedToken;
use std::time::{Duration, Instant};

fn create_large_document() -> String {
    let mut doc = String::new();
    for i in 0..100 {
        doc.push_str(&format!("# Section {}\n\n", i));
        for j in 0..20 {
            doc.push_str(&format!(
                "This is paragraph {} in section {}. It contains several sentences. \
                 Each sentence has multiple words. Some words are longer than others. \
                 This helps test the timing modifiers. Here's a question? And an exclamation!\n\n",
                j, i
            ));
        }
    }
    doc
}

#[test]
fn test_render_performance_at_positions() {
    let markdown = create_large_document();
    let parser = MarkdownParser::new();
    let doc = parser.parse_str(&markdown).unwrap();

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

    println!("\nTotal tokens: {}", timed_tokens.len());

    let mut app = App::new(timed_tokens, doc.sections);

    // Test positions: start, 25%, 50%, 75%, near end
    let total = app.tokens().len();
    let test_positions = [0, total / 4, total / 2, total * 3 / 4, total - 100];

    // Simulate what compute_document_lines does
    for &pos in &test_positions {
        // Set position by advancing
        while app.position() < pos {
            app.advance();
        }

        let tokens = app.tokens();
        let start = pos.saturating_sub(500);
        let end = (pos + 500).min(tokens.len());

        // Measure the iteration time
        let iterations = 100;
        let start_time = Instant::now();

        for _ in 0..iterations {
            let mut count = 0usize;
            for (idx, token) in (start..end).zip(&tokens[start..end]) {
                // Simulate the work done in compute_document_lines
                let _ = token.token.word.chars().count();
                let _ = idx;
                count += 1;
            }
            std::hint::black_box(count);
        }

        let elapsed = start_time.elapsed();
        let per_iter_us = elapsed.as_micros() as f64 / iterations as f64;

        println!(
            "Position {:>6} ({:>3}%): {:>6.1}µs per iteration, range {}..{} ({} tokens)",
            pos,
            pos * 100 / total,
            per_iter_us,
            start,
            end,
            end - start
        );
    }
}

#[test]
fn test_actual_timing_at_positions() {
    // This test measures actual elapsed time between words at different positions
    // to detect if later positions take longer
    let markdown = create_large_document();
    let parser = MarkdownParser::new();
    let doc = parser.parse_str(&markdown).unwrap();

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

    let total = timed_tokens.len();
    println!("\nTotal tokens: {}", total);

    let mut app = App::new(timed_tokens, doc.sections);

    // Measure time to advance through 100 words at different positions
    let test_positions = [0, total / 4, total / 2, total * 3 / 4];
    let words_to_advance = 100;

    for &start_pos in &test_positions {
        // Reset to start position
        while app.position() < start_pos {
            app.advance();
        }

        // Measure time to advance through words
        let mut total_duration = Duration::ZERO;
        let mut expected_duration = Duration::ZERO;

        for _ in 0..words_to_advance {
            if let Some(token) = app.current_token() {
                let expected_ms = calculate_duration(&token.token, wpm);
                expected_duration += Duration::from_millis(expected_ms);
            }

            let start = Instant::now();
            // Simulate what happens each frame: calculate duration
            if let Some(token) = app.current_token() {
                let _ = calculate_duration(&token.token, app.wpm());
            }
            app.advance();
            total_duration += start.elapsed();
        }

        println!(
            "Position {:>6} ({:>3}%): advance overhead {:>6.1}µs/word, expected duration {:>3}ms/word",
            start_pos,
            start_pos * 100 / total,
            total_duration.as_micros() as f64 / words_to_advance as f64,
            expected_duration.as_millis() / words_to_advance as u128,
        );
    }
}

#[test]
fn test_skip_vs_slice_performance() {
    let markdown = create_large_document();
    let parser = MarkdownParser::new();
    let doc = parser.parse_str(&markdown).unwrap();

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

    let total = timed_tokens.len();
    println!(
        "\nComparing skip() vs slice indexing at position {}",
        total - 100
    );

    let pos = total - 100;
    let start = pos.saturating_sub(500);
    let end = (pos + 500).min(total);
    let iterations = 1000;

    // Test with enumerate().skip()
    let start_time = Instant::now();
    for _ in 0..iterations {
        let mut count = 0usize;
        for (idx, token) in timed_tokens
            .iter()
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            let _ = token.token.word.chars().count();
            let _ = idx;
            count += 1;
        }
        std::hint::black_box(count);
    }
    let skip_elapsed = start_time.elapsed();

    // Test with slice indexing
    let start_time = Instant::now();
    for _ in 0..iterations {
        let mut count = 0usize;
        for (idx, token) in (start..end).zip(&timed_tokens[start..end]) {
            let _ = token.token.word.chars().count();
            let _ = idx;
            count += 1;
        }
        std::hint::black_box(count);
    }
    let slice_elapsed = start_time.elapsed();

    println!(
        "enumerate().skip(): {:>6.1}µs per iteration",
        skip_elapsed.as_micros() as f64 / iterations as f64
    );
    println!(
        "slice indexing:     {:>6.1}µs per iteration",
        slice_elapsed.as_micros() as f64 / iterations as f64
    );
    println!(
        "Speedup: {:.1}x",
        skip_elapsed.as_micros() as f64 / slice_elapsed.as_micros() as f64
    );
}
