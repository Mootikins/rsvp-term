use insta::assert_snapshot;
use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};
use rsvp_term::app::App;
use rsvp_term::parser::traits::DocumentParser;
use rsvp_term::parser::MarkdownParser;
use rsvp_term::types::{BlockContext, TimedToken, TimingHint, Token, TokenStyle};

fn make_timed_token(word: &str) -> TimedToken {
    TimedToken {
        token: Token {
            word: word.to_string(),
            style: TokenStyle::Normal,
            block: BlockContext::Paragraph,
            parent_context: None,
            timing_hint: TimingHint::default(),
        },
        duration_ms: 200,
        orp_position: 1,
    }
}

fn create_long_test_app() -> App {
    // Create a much longer document that will span multiple lines even in a wide terminal
    let words = "The quick brown fox jumps over the lazy dog and then runs around the park \
        while the sun shines brightly in the clear blue sky above the green meadow where \
        birds sing their songs and flowers bloom in vibrant colors creating a beautiful \
        scene that captivates all who witness it and brings joy to their hearts as they \
        walk along the winding path through the enchanted forest filled with ancient trees \
        and mysterious creatures that have lived there for centuries watching over the land \
        with wisdom and grace that transcends time itself and connects all living things \
        in a web of life that stretches across the entire universe binding us together \
        in ways we cannot fully comprehend but can feel deep within our souls";
    let tokens: Vec<TimedToken> = words.split_whitespace().map(make_timed_token).collect();
    App::new(tokens, vec![])
}

fn render_to_string(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            rsvp_term::ui::render(frame, app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    buffer_to_string(buffer)
}

fn buffer_to_string(buffer: &Buffer) -> String {
    let mut result = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = buffer.cell((x, y)).unwrap();
            result.push_str(cell.symbol());
        }
        result.push('\n');
    }
    result
}

/// Test that context lines don't reflow as we advance through words
/// in a large terminal with a long document
#[test]
fn test_context_no_reflow_on_advance() {
    let mut app = create_long_test_app();

    // Use a large terminal
    let width = 160;
    let height = 50;

    // Take snapshot at position 0
    let output_pos0 = render_to_string(&app, width, height);
    assert_snapshot!("large_term_pos0", output_pos0);

    // Advance to position 20
    for _ in 0..20 {
        app.advance();
    }
    let output_pos20 = render_to_string(&app, width, height);
    assert_snapshot!("large_term_pos20", output_pos20);

    // Advance to position 40
    for _ in 0..20 {
        app.advance();
    }
    let output_pos40 = render_to_string(&app, width, height);
    assert_snapshot!("large_term_pos40", output_pos40);

    // Advance to position 60
    for _ in 0..20 {
        app.advance();
    }
    let output_pos60 = render_to_string(&app, width, height);
    assert_snapshot!("large_term_pos60", output_pos60);
}

#[test]
fn test_list_items_have_structure_modifier_on_first_word() {
    let parser = MarkdownParser::new();
    let doc = parser
        .parse_str("- Item one\n- Item two\n- Item three")
        .unwrap();

    let first_words: Vec<_> = doc
        .tokens
        .iter()
        .filter(|t| matches!(t.block, BlockContext::ListItem(_)))
        .filter(|t| t.timing_hint.structure_modifier > 0)
        .map(|t| t.word.as_str())
        .collect();

    assert_eq!(first_words, vec!["Item", "Item", "Item"]);
}

#[test]
fn test_list_items_should_be_separate_lines() {
    let parser = MarkdownParser::new();
    let doc = parser
        .parse_str("- First item\n- Second item\n- Third item")
        .unwrap();

    let new_line_triggers: Vec<_> = doc
        .tokens
        .iter()
        .filter(|t| {
            matches!(t.block, BlockContext::ListItem(_)) && t.timing_hint.structure_modifier > 0
        })
        .collect();

    assert_eq!(new_line_triggers.len(), 3);
}
