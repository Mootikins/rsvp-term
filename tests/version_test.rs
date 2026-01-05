use std::process::Command;

#[test]
fn test_version_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_rsvp-term"))
        .arg("--version")
        .output()
        .expect("Failed to execute rsvp-term");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should exit successfully and print version
    assert!(output.status.success(), "rsvp-term --version should exit successfully");
    assert!(stdout.contains("rsvp-term"), "Version output should contain 'rsvp-term'");
    assert!(stdout.contains("0.3.0"), "Version output should contain version number");
}
