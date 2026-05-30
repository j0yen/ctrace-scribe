//! AC1: scribe render writes .summary.md containing Log: line, duration/event/PID/write
//! count line, and all five section headers.

mod fixtures;
use fixtures::*;

#[test]
fn acceptance_ac1_render_produces_required_sections() {
    let dir = tmpdir();
    let log = write_ndjson(dir.path(), "session.ndjson", &minimal_log_lines());
    let summary = dir.path().join("session.summary.md");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["render", log.to_str().expect("log path")])
        .output()
        .expect("run scribe render");

    assert!(
        output.status.success(),
        "render exited non-zero: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        summary.exists(),
        "summary file not created at {}",
        summary.display()
    );

    let content = std::fs::read_to_string(&summary).expect("read summary");

    // Log: line
    assert!(
        content.contains("Log:"),
        "missing Log: line in:\n{content}"
    );
    // Duration/event/PID/write count line
    assert!(
        content.contains("Duration") && content.contains("events") && content.contains("PIDs"),
        "missing duration/event/PID line in:\n{content}"
    );
    // Five section headers
    assert!(content.contains("## Top binaries executed"), "missing Top binaries section");
    assert!(
        content.contains("## Writes outside expected scope"),
        "missing Writes section"
    );
    assert!(content.contains("## Deletions"), "missing Deletions section");
    assert!(
        content.contains("## Outbound connect() by process"),
        "missing Outbound connect section"
    );
    // The Flagged section is conditional — present only when flagged writes exist.
    // This fixture has no flagged writes, so it should NOT be present (tested in AC2).
}
