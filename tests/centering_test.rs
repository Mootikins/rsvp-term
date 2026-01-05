use insta::assert_debug_snapshot;

/// Test data for centering calculations
#[derive(Debug)]
struct CenteringCase {
    content_width: usize,
    available_width: usize,
    expected_padding: usize,
    description: &'static str,
}

const CENTER_THRESHOLD: f32 = 0.6;
const MIN_PADDING: usize = 2;

fn calculate_padding(content_width: usize, available_width: usize) -> usize {
    let ratio = content_width as f32 / available_width as f32;

    if ratio < CENTER_THRESHOLD {
        (available_width.saturating_sub(content_width)) / 2
    } else {
        MIN_PADDING
    }
    .max(MIN_PADDING)
}

fn generate_visual(content: &str, available_width: usize) -> String {
    let content_width = content.len();
    let padding = calculate_padding(content_width, available_width);
    let spaces = " ".repeat(padding);
    format!("|{}{}|", spaces, content)
}

#[test]
fn test_centering_scenarios() {
    let cases = vec![
        CenteringCase {
            content_width: 10,
            available_width: 80,
            expected_padding: 35, // (80-10)/2
            description: "Short heading in wide terminal",
        },
        CenteringCase {
            content_width: 20,
            available_width: 80,
            expected_padding: 30, // (80-20)/2
            description: "Medium heading in wide terminal",
        },
        CenteringCase {
            content_width: 50,
            available_width: 80,
            expected_padding: 2, // > 60%, left-aligned
            description: "Long line exceeds threshold",
        },
        CenteringCase {
            content_width: 47,
            available_width: 80,
            expected_padding: 16, // just under 60%
            description: "Just under threshold",
        },
        CenteringCase {
            content_width: 48,
            available_width: 80,
            expected_padding: 2, // exactly 60%
            description: "Exactly at threshold",
        },
    ];

    let results: Vec<_> = cases
        .iter()
        .map(|c| {
            let actual = calculate_padding(c.content_width, c.available_width);
            (c.description, c.content_width, c.available_width, actual)
        })
        .collect();

    assert_debug_snapshot!(results);
}

#[test]
fn test_visual_centering_examples() {
    let examples = vec![
        ("# Title", 80),
        ("## Subtitle", 80),
        ("The quick brown fox jumps over the lazy dog.", 80),
        ("Short.", 60),
        ("A very long paragraph that definitely exceeds the centering threshold and should be left-aligned.", 80),
    ];

    let visuals: Vec<_> = examples
        .iter()
        .map(|(content, width)| {
            let visual = generate_visual(content, *width);
            (content, width, visual)
        })
        .collect();

    assert_debug_snapshot!(visuals);
}

#[test]
fn test_narrow_terminal_centering() {
    // In narrow terminals, even short content might exceed threshold
    let cases = vec![
        (10, 40), // 25% - centered
        (20, 40), // 50% - centered
        (25, 40), // 62.5% - left-aligned
        (30, 40), // 75% - left-aligned
    ];

    let results: Vec<_> = cases
        .iter()
        .map(|(content, width)| {
            let padding = calculate_padding(*content, *width);
            (*content, *width, padding)
        })
        .collect();

    assert_debug_snapshot!(results);
}
