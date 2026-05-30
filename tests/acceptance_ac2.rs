//! AC2: Flagged section is omitted when no flagged writes; present when /home/jsy/.ssh/ write exists.

mod fixtures;
use fixtures::*;

#[test]
fn acceptance_ac2_no_flagged_section_when_no_sensitive_writes() {
    let dir = tmpdir();
    let log = write_ndjson(dir.path(), "session_noflag.ndjson", &no_flagged_log_lines());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["render", log.to_str().expect("log path")])
        .output()
        .expect("run scribe render");

    assert!(output.status.success(), "render failed");

    let summary = dir.path().join("session_noflag.summary.md");
    let content = std::fs::read_to_string(&summary).expect("read summary");

    assert!(
        !content.contains("Flagged sensitive-path"),
        "Flagged section should be absent but found in:\n{content}"
    );
}

#[test]
fn acceptance_ac2_flagged_section_present_for_ssh_write() {
    let dir = tmpdir();
    let log = write_ndjson(dir.path(), "session_flagged.ndjson", &flagged_ssh_log_lines());

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_scribe"))
        .args(["render", log.to_str().expect("log path")])
        .output()
        .expect("run scribe render");

    assert!(output.status.success(), "render failed");

    let summary = dir.path().join("session_flagged.summary.md");
    let content = std::fs::read_to_string(&summary).expect("read summary");

    assert!(
        content.contains("Flagged sensitive-path"),
        "Flagged section should be present for SSH write in:\n{content}"
    );
    assert!(
        content.contains(".ssh/known_hosts"),
        "Flagged section should list .ssh/known_hosts in:\n{content}"
    );
}
