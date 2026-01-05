use std::process::Command;

#[test]
fn test_no_hint_chars_flag_accepted() {
    let output = Command::new(env!("CARGO_BIN_EXE_rsvp-term"))
        .args(["--no-hint-chars", "--help"])
        .output()
        .expect("Failed to run");
    assert!(output.status.success());
}

#[test]
fn test_no_styling_flag_accepted() {
    let output = Command::new(env!("CARGO_BIN_EXE_rsvp-term"))
        .args(["--no-styling", "--help"])
        .output()
        .expect("Failed to run");
    assert!(output.status.success());
}
